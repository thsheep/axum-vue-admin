use crate::{bad_request, conflict, errors::app_error::AppError, forbidden, not_found};
use cedar_policy::{Entities, Entity, EntityId, EntityTypeName, EntityUid, RestrictedExpression};
use sea_orm::{
    ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QuerySelect, TransactionTrait,
    entity::prelude::*,
};
use std::collections::{HashMap, HashSet, VecDeque};
use std::str::FromStr;

use crate::config::state::AppState;
use crate::entity::{departments, users};
use crate::schemas::auth::CurrentUser;
use crate::schemas::cedar_policy::CedarContext;
use crate::schemas::department::{CreateDepartmentDto, DepartmentResponse, DeptTreeNode};
use crate::schemas::user::UserResponse;
use crate::utils::cedar_utils::{entities2json, AuthAction, ResourceType, ENTITY_TYPE_DEPARTMENT};
use crate::utils::services::{assemble_user_info, build_dept_tree};
use sea_orm::ActiveValue::Set;
use sea_orm::JoinType::InnerJoin;
use tracing::debug;

const MAX_DEPT_DEPTH: usize = 100;
const ROOT_DEPARTMENT_ID: i32 = 0;

#[derive(Clone)]
pub struct DepartmentService {
    app_state: AppState,
}

impl DepartmentService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    pub async fn get_dept_entities(&self, dept_id: i32) -> Result<Entities, AppError> {
        let dept = departments::Entity::find_by_id(dept_id)
            .one(&self.app_state.db)
            .await?;

        let dept = match dept {
            Some(dept) => dept,
            None => return Ok(Entities::empty()),
        };

        let dept_eid = EntityId::from_str(&dept.dept_id.to_string())?;
        let dept_typename = EntityTypeName::from_str("Department")?;
        let dept_e_uid = EntityUid::from_type_name_and_id(dept_typename, dept_eid);

        let mut attrs = HashMap::new();
        let name_expr = RestrictedExpression::new_string(dept.name);
        attrs.insert("name".to_string(), name_expr);

        let parents = HashSet::new();
        let dept_entity = Entity::new(dept_e_uid, attrs, parents)?;

        let schema = self.app_state.auth_service.get_schema_copy().await;
        let verified_entities = Entities::from_entities(vec![dept_entity], Some(&schema))?;
        let entities_json = entities2json(&verified_entities)?;
        debug!("Dept:{:?}; Entities Json: {}", dept_id, entities_json);
        Ok(verified_entities)
    }

    // 验证部门ID是否存在
    pub async fn validate_dept_id(&self, dept_id: i32) -> Result<(), AppError> {
        let exists = departments::Entity::find_by_id(dept_id)
            .one(&self.app_state.db)
            .await?
            .is_some();

        if !exists {
            return Err(bad_request!("Department does not exist".to_string()));
        }
        Ok(())
    }

    pub async fn list_departments(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
    ) -> Result<Vec<DeptTreeNode>, AppError> {
        self.app_state
            .auth_service
            .check_permission(
                current_user.clone(),
                context,
                AuthAction::ViewDepartment,
                ResourceType::Department(None),
            )
            .await?;

        let all_departments = departments::Entity::find()
            .filter(departments::Column::IsDeleted.eq(false))
            .all(&self.app_state.db)
            .await?;

        if current_user.is_super_admin {
            return Ok(build_dept_tree(&all_departments, 0).await?);
        };

        // let user_dept_id = users::Entity::find()
        //     .column(users::Column::DeptId)
        //     .filter(users::Column::UserId.eq(current_user.user_id))
        //     .one(&self.app_state.db)
        //     .await?
        //     .and_then(|user| user.dept_id)
        //     .ok_or(not_found!("User department not found".to_string()))?;

        let dept_parent_id = departments::Entity::find()
            .column(departments::Column::ParentId)
            .join(InnerJoin, departments::Relation::Users.def())
            .filter(users::Column::UserId.eq(current_user.user_id))
            .one(&self.app_state.db)
            .await?
            .map(|m| {m.parent_id})
            .ok_or(not_found!("User department not found".to_string()))?;


        Ok(build_dept_tree(&all_departments, dept_parent_id).await?)
    }

    pub async fn create_department(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        dto: CreateDepartmentDto,
    ) -> Result<DepartmentResponse, AppError> {
        self.app_state
            .auth_service
            .check_permission(
                current_user,
                context,
                AuthAction::CreateDepartment,
                ResourceType::Department(None),
            )
            .await?;

        if departments::Entity::find()
            .filter(
                departments::Column::Name
                    .eq(&dto.name)
                    .and(departments::Column::ParentId.eq(dto.parent_id)),
            )
            .one(&self.app_state.db)
            .await?
            .is_some()
        {
            return Err(conflict!("department name already exists".to_string(),));
        };

        let department = departments::ActiveModel {
            name: Set(dto.name),
            desc: Set(Some(dto.desc)),
            order: Set(dto.order),
            parent_id: Set(dto.parent_id as i32),
            ..Default::default()
        };

        let department = department.insert(&self.app_state.db).await?;
        Ok(DepartmentResponse::from(department))
    }

    pub async fn update_department(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        dept_id: i32,
        dto: CreateDepartmentDto,
    ) -> Result<DepartmentResponse, AppError> {
        let es = self.get_dept_entities(dept_id).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                current_user.clone(),
                context.clone(),
                AuthAction::UpdateDepartment,
                ResourceType::Department(Some(dept_id)),
                es,
            )
            .await?;

        let txn = self.app_state.db.begin().await?;

        let mut department: departments::ActiveModel = departments::Entity::find_by_id(dept_id)
            .one(&txn)
            .await?
            .ok_or(not_found!("department not found".to_string()))?
            .into();

        department.name = Set(dto.name);
        department.desc = Set(Some(dto.desc));
        department.order = Set(dto.order);

        let original_parent_id = department.parent_id.as_ref().clone();
        if original_parent_id != dto.parent_id {
            self.app_state
                .auth_service
                .check_permission(
                    current_user.clone(),
                    context,
                    AuthAction::MoveDepartment,
                    ResourceType::Department(Some(dept_id)),
                )
                .await?;
        }

        department.parent_id = Set(dto.parent_id);

        let department = department.update(&txn).await?;
        txn.commit().await?;
        Ok(DepartmentResponse::from(department))
    }

    pub async fn delete_department(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        dept_id: i32,
    ) -> Result<(), AppError> {
        let es = self.get_dept_entities(dept_id).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                current_user,
                context,
                AuthAction::DeleteDepartment,
                ResourceType::Department(Some(dept_id)),
                es,
            )
            .await?;

        let txn = self.app_state.db.begin().await?;

        let department = departments::Entity::find_by_id(dept_id)
            .columns([departments::Column::DeptId, departments::Column::ParentId])
            .one(&txn)
            .await?
            .ok_or(not_found!("department not found".to_string()))?;

        if department.parent_id == ROOT_DEPARTMENT_ID {
            return Err(bad_request!(
                "Deletion of department root node is not allowed".to_string(),
            ));
        }
        // 检查部门是否有子部门
        if self
            .has_children_optimized(&txn, department.dept_id)
            .await?
        {
            return Err(bad_request!(
                "Department has children, deletion not allowed".to_string(),
            ));
        }

        // 检查该部门是否还有用户
        if self.has_user_optimized(&txn, department.dept_id).await? {
            return Err(bad_request!(
                "Department has users, deletion not allowed".to_string(),
            ));
        }
        // 如果是这个部门还绑定了其他资源，还需要补全逻辑

        // 删除部门
        let dept = departments::ActiveModel {
            dept_id: Set(department.dept_id),
            is_deleted: Set(true),
            ..Default::default()
        };
        dept.update(&txn).await?;

        txn.commit().await?;
        Ok(())
    }

    async fn has_children_optimized(
        &self,
        txn: &DatabaseTransaction,
        department_id: i32,
    ) -> Result<bool, AppError> {
        let child_count = departments::Entity::find()
            .filter(
                departments::Column::ParentId
                    .eq(department_id)
                    .and(departments::Column::IsDeleted.eq(false)),
            )
            .count(txn)
            .await?;
        Ok(child_count > 0)
    }

    async fn has_user_optimized(
        &self,
        txn: &DatabaseTransaction,
        department_id: i32,
    ) -> Result<bool, AppError> {
        let user_count = users::Entity::find()
            .filter(users::Column::DeptId.eq(department_id))
            .count(txn)
            .await?;
        Ok(user_count > 0)
    }

    pub async fn department_users(&self,
                                  current_user: CurrentUser,
                                  context: CedarContext,
                                  dept_id: i32) -> Result<Vec<UserResponse>, AppError> {

        let es = self.get_dept_entities(dept_id).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                current_user,
                context,
                AuthAction::ViewDepartmentUsers,
                ResourceType::Department(Some(dept_id)),
                es,
            )
            .await?;

        let users_with_dept = users::Entity::find()
            .find_also_related(departments::Entity)
            .filter(users::Column::DeptId.eq(dept_id))
            .all(&self.app_state.db)
            .await?;
        let users = assemble_user_info(&self.app_state.db, users_with_dept).await?;
        Ok(users)
    }

    // 获取指定部门的子部门
    pub async fn find_descendants_entities(&self, dept_id: i32) -> Result<Entities, AppError> {
        // 1. 一次性获取所有未被软删除的部门
        let all_depts: Vec<departments::Model> = departments::Entity::find()
            .filter(departments::Column::IsDeleted.eq(false))
            .all(&self.app_state.db)
            .await?;

        // 2. 按父ID对部门进行分组，方便查找子部门
        let depts_by_parent: HashMap<i32, Vec<&departments::Model>> =
            all_depts.iter().fold(HashMap::new(), |mut acc, dept| {
                acc.entry(dept.parent_id).or_default().push(dept);
                acc
            });

        // 3. 按部门ID创建索引，方便通过ID快速查找部门model
        let depts_by_id: HashMap<i32, &departments::Model> =
            all_depts.iter().map(|d| (d.dept_id, d)).collect();

        // 3. 初始化遍历所需的数据结构
        // 检查起始部门是否存在于我们获取的列表中
        if !depts_by_id.contains_key(&dept_id) {
            return Ok(Entities::empty()); // 如果起始部门不存在或已被删除，返回空结果
        }

        let mut entities = HashSet::new(); // 使用 HashSet 存储最终的 Cedar 实体，自动去重
        let mut ids_to_process: VecDeque<i32> = VecDeque::new();
        let mut processed_ids: HashSet<i32> = HashSet::new(); // 防止因数据循环引用导致无限循环

        // 4. 开始广度优先搜索 (BFS) 遍历
        ids_to_process.push_back(dept_id);

        while let Some(current_dept_id) = ids_to_process.pop_front() {
            // 如果已处理过此ID，则跳过
            if !processed_ids.insert(current_dept_id) {
                continue;
            }

            // 从索引中获取当前部门的模型，并将其转换为Cedar实体
            if let Some(current_dept_model) = depts_by_id.get(&current_dept_id) {
                let entity = self.try_dept_model_to_cedar_entity(current_dept_model)?;
                entities.insert(entity);

                // 查找并添加所有直接子部门到处理队列中
                if let Some(children) = depts_by_parent.get(&current_dept_id) {
                    for child in children {
                        // 仅添加未处理过的子部门ID
                        if !processed_ids.contains(&child.dept_id) {
                            ids_to_process.push_back(child.dept_id);
                        }
                    }
                }
            }
        }
        // 5. 从实体集合创建最终的 `Entities` 对象
        Entities::from_entities(entities, None).map_err(AppError::from)
    }

    fn try_dept_model_to_cedar_entity(
        &self,
        dept: &departments::Model,
    ) -> Result<Entity, AppError> {
        let dept_eid = EntityId::from_str(&dept.dept_id.to_string())?;
        let dept_typename = EntityTypeName::from_str(ENTITY_TYPE_DEPARTMENT)?;
        let dept_e_uid = EntityUid::from_type_name_and_id(dept_typename, dept_eid);

        let mut attrs = HashMap::new();
        let name_expr = RestrictedExpression::new_string(dept.name.clone());
        attrs.insert("name".to_string(), name_expr);

        // Cedar 实体中的 parents 指的是其所属的组，这里为空是合理的
        let parents = HashSet::new();
        let dept_entity = Entity::new(dept_e_uid, attrs, parents)?;
        Ok(dept_entity)
    }
}
