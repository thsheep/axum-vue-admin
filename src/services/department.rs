use crate::{bad_request, conflict, errors::app_error::AppError, forbidden, not_found};
use cedar_policy::{Entities, Entity, EntityId, EntityTypeName, EntityUid, RestrictedExpression, Schema};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QuerySelect, TransactionTrait, entity::prelude::*, Statement, DbBackend, TryGetableMany, JoinType, Condition, QueryOrder};
use std::collections::{HashMap, HashSet, VecDeque};
use std::str::FromStr;
use async_recursion::async_recursion;
use crate::config::state::AppState;
use crate::entity::{departments, user_group_members, user_groups, users};
use crate::schemas::auth::CurrentUser;
use crate::schemas::cedar_policy::CedarContext;
use crate::schemas::department::{CreateDepartmentDto, DepartmentResponse, DeptTreeNode};
use crate::schemas::user::{DeptResponse, GroupResponse, UserResponse};
use crate::utils::cedar_utils::{entities2json, AuthAction, ResourceType, ENTITY_TYPE_DEPARTMENT};
use sea_orm::ActiveValue::Set;
use tracing::{debug, warn};

const MAX_DEPT_DEPTH: usize = 100;
const ROOT_DEPARTMENT_ID: i32 = 0;
const ROOT_DEPARTMENT_UUID: &str = "0";

#[derive(Clone)]
pub struct DepartmentService {
    app_state: AppState,
}

impl DepartmentService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn get_dept_id_from_uuid(
        &self,
        db: &impl ConnectionTrait,
        dept_uuid: &str,
    ) -> Result<i32, AppError> {
        departments::Entity::find()
            .select_only()
            .column(departments::Column::DeptId)
            .filter(departments::Column::DeptUuid.eq(dept_uuid))
            .into_tuple::<i32>()
            .one(db)
            .await?
            .ok_or(not_found!(format!("Department with UUID '{}' not found", dept_uuid)))
    }

    pub async fn list_departments(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
    ) -> Result<Vec<DeptTreeNode>, AppError> {
        self.app_state
            .auth_service
            .check_permission(
                &current_user.uuid,
                context,
                AuthAction::ViewDepartment,
                ResourceType::Department(None),
            )
            .await?;

        let all_departments = departments::Entity::find()
            .filter(departments::Column::IsDeleted.eq(false))
            .all(&self.app_state.db)
            .await?;

        let root_parent_id = if current_user.is_super_admin {
            ROOT_DEPARTMENT_ID
        } else {
            let user_dept_id = users::Entity::find()
                .select_only()
                .column(users::Column::DeptId)
                .filter(users::Column::UserUuid.eq(&current_user.uuid))
                .into_tuple::<i32>()
                .one(&self.app_state.db)
                .await?
                .ok_or_else(|| not_found!("User's current department not found"))?;

            // 从完整列表中找到该部门的 parent_id
            all_departments
                .iter()
                .find(|d| d.dept_id == user_dept_id)
                .map(|d| d.parent_id)
                .unwrap_or(ROOT_DEPARTMENT_ID) // 如果找不到，退回到根
        };

        let tree_node = build_dept_tree_optimized_with_uuid(&all_departments, root_parent_id).await?;
        Ok(tree_node)
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
                &current_user.uuid,
                context,
                AuthAction::CreateDepartment,
                ResourceType::Department(None),
            )
            .await?;

        let parent_id = if dto.parent_uuid == ROOT_DEPARTMENT_UUID {
            ROOT_DEPARTMENT_ID
        } else {
            self.get_dept_id_from_uuid(&self.app_state.db, &dto.parent_uuid).await?
        };

        if departments::Entity::find()
            .filter(
                Condition::all()
                    .add(departments::Column::Name.eq(&dto.name))
                    .add(departments::Column::ParentId.eq(parent_id))
                    .add(departments::Column::IsDeleted.eq(false))
            )
            .one(&self.app_state.db)
            .await?
            .is_some()
        {
            return Err(conflict!("A department with the same name already exists under this parent."));
        };

        let new_department = departments::ActiveModel {
            dept_uuid: Set(Uuid::new_v4().to_string()),
            name: Set(dto.name),
            desc: Set(Some(dto.desc)),
            order: Set(dto.order),
            parent_id: Set(parent_id),
            ..Default::default()
        };

        let saved_department = new_department.insert(&self.app_state.db).await?;

        Ok(DepartmentResponse {
            uuid: saved_department.dept_uuid,
            name: saved_department.name,
            desc: saved_department.desc,
            order: saved_department.order,
            parent_uuid: dto.parent_uuid,
        })
    }

    pub async fn update_department(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        dept_uuid: String,
        dto: CreateDepartmentDto,
    ) -> Result<DepartmentResponse, AppError> {
        let schema = self.app_state.auth_service.get_schema_copy().await;
        let es = get_dept_entities(&self.app_state.db, &dept_uuid, &schema).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                &current_user.uuid,
                context.clone(),
                AuthAction::UpdateDepartment,
                ResourceType::Department(Some(dept_uuid.clone())),
                es,
            )
            .await?;

        let txn = self.app_state.db.begin().await?;

        let new_parent_id = if dto.parent_uuid == ROOT_DEPARTMENT_UUID {
            ROOT_DEPARTMENT_ID
        } else {
            self.get_dept_id_from_uuid(&txn, &dto.parent_uuid).await?
        };

        let mut department: departments::ActiveModel = departments::Entity::find()
            .filter(departments::Column::DeptUuid.eq(&dept_uuid))
            .one(&txn)
            .await?
            .ok_or(not_found!("department not found".to_string()))?
            .into();

        let original_parent_id = *department.parent_id.as_ref();
        if original_parent_id != new_parent_id {
            self.app_state
                .auth_service
                .check_permission(
                    &current_user.uuid,
                    context,
                    AuthAction::MoveDepartment,
                    ResourceType::Department(Some(dept_uuid)),
                )
                .await?;
        }

        department.name = Set(dto.name);
        department.desc = Set(Some(dto.desc));
        department.order = Set(dto.order);
        department.parent_id = Set(new_parent_id);

        let updated_department = department.update(&txn).await?;
        txn.commit().await?;
        Ok(DepartmentResponse {
            uuid: updated_department.dept_uuid,
            name: updated_department.name,
            desc: updated_department.desc,
            order: updated_department.order,
            parent_uuid: dto.parent_uuid,
        })
    }

    pub async fn delete_department(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        dept_uuid: String,
    ) -> Result<(), AppError> {
        let schema = self.app_state.auth_service.get_schema_copy().await;
        let es = get_dept_entities(&self.app_state.db, &dept_uuid, &schema).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                &current_user.uuid,
                context,
                AuthAction::DeleteDepartment,
                ResourceType::Department(Some(dept_uuid.clone())),
                es,
            )
            .await?;

        let txn = self.app_state.db.begin().await?;

        let dept_model = departments::Entity::find()
            .filter(departments::Column::DeptUuid.eq(&dept_uuid))
            .one(&txn)
            .await?
            .ok_or_else(|| not_found!("Department to delete not found"))?;


        if dept_model.parent_id == ROOT_DEPARTMENT_ID {
            return Err(bad_request!("Deletion of root department is not allowed."));
        };

        // 检查部门是否有子部门
        if self.has_children_optimized(&txn, dept_model.dept_id).await? {
            return Err(bad_request!("Department has children, deletion not allowed."));
        }

        // 检查该部门是否还有用户
        if self.has_user_optimized(&txn, dept_model.dept_id).await? {
            return Err(bad_request!("Department has users, deletion not allowed."));
        }
        // 如果是这个部门还绑定了其他资源，还需要补全逻辑

        // 删除部门
        let mut active_model: departments::ActiveModel = dept_model.into();
        active_model.is_deleted = Set(true);
        active_model.update(&txn).await?;

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
                                  dept_uuid: String) -> Result<Vec<UserResponse>, AppError> {

        let schema = self.app_state.auth_service.get_schema_copy().await;
        let es = get_dept_entities(&self.app_state.db, &dept_uuid, &schema).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                &current_user.uuid,
                context,
                AuthAction::ViewDepartmentUsers,
                ResourceType::Department(Some(dept_uuid.clone())),
                es,
            )
            .await?;

        let users_with_dept = users::Entity::find()
            .find_also_related(departments::Entity)
            .join(
                JoinType::InnerJoin,
                users::Relation::Departments.def(),
            )
            .filter(departments::Column::DeptUuid.eq(dept_uuid))
            .all(&self.app_state.db)
            .await?;
        let users = assemble_user_info(&self.app_state.db, users_with_dept).await?;
        Ok(users)
    }
}


pub async fn get_all_child_dept_ids(
    db: &DatabaseConnection,
    parent_dept_uuid: &str,
) -> Result<Vec<i32>, AppError> {
    let parent_dept_id = departments::Entity::find()
        .select_only()
        .column(departments::Column::DeptId)
        .filter(departments::Column::DeptUuid.eq(parent_dept_uuid))
        .into_tuple::<i32>()
        .one(db)
        .await?
        .ok_or(not_found!("department not found".to_string()))?;

    let all_depts = departments::Entity::find()
        .filter(departments::Column::IsDeleted.eq(false))
        .all(db)
        .await?;

    let depts_by_parent: HashMap<i32, Vec<departments::Model>> =
        all_depts.into_iter().fold(HashMap::new(), |mut acc, dept| {
            acc.entry(dept.parent_id).or_default().push(dept);
            acc
        });

    let mut all_child_ids = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(parent_dept_id);

    // 包含起始的父部门ID
    all_child_ids.push(parent_dept_id);

    while let Some(current_id) = queue.pop_front() {
        if let Some(children) = depts_by_parent.get(&current_id) {
            for child in children {
                if !all_child_ids.contains(&child.dept_id) {
                    all_child_ids.push(child.dept_id);
                    queue.push_back(child.dept_id);
                }
            }
        }
    }

    Ok(all_child_ids)
}


// 获取指定部门的Entities

pub async fn get_dept_entities(db: &DatabaseConnection, dept_uuid: &str, schema: &Schema) -> Result<Entities, AppError> {
    let dept_name = departments::Entity::find()
        .select_only()
        .column(departments::Column::Name)
        .filter(departments::Column::DeptUuid.eq(dept_uuid))
        .into_tuple::<String>()
        .one(db)
        .await?;

    let dept_name = match dept_name {
        Some(dept_name) => dept_name,
        None => return Ok(Entities::empty()),
    };

    let dept_eid = EntityId::from_str(dept_uuid.as_ref())?;
    let dept_typename = EntityTypeName::from_str(ENTITY_TYPE_DEPARTMENT)?;
    let dept_e_uid = EntityUid::from_type_name_and_id(dept_typename, dept_eid);

    let mut attrs = HashMap::new();
    let name_expr = RestrictedExpression::new_string(dept_name);
    attrs.insert("name".to_string(), name_expr);

    let parents = HashSet::new();
    let dept_entity = Entity::new(dept_e_uid, attrs, parents)?;

    let verified_entities = Entities::from_entities(vec![dept_entity], Some(&schema))?;
    let entities_json = entities2json(&verified_entities)?;
    debug!("Dept:{:?}; Entities Json: {}", dept_uuid, entities_json);
    Ok(verified_entities)
}

// 获取指定部门所有的子部门Entities
pub async fn find_descendants_entities(db: &DatabaseConnection, dept_id: i32) -> Result<Entities, AppError> {
    // 1. 一次性获取所有未被软删除的部门
    let all_depts: Vec<departments::Model> = departments::Entity::find()
        .filter(departments::Column::IsDeleted.eq(false))
        .all(db)
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
            let entity = try_dept_model_to_cedar_entity(current_dept_model)?;
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
    dept: &departments::Model,
) -> Result<Entity, AppError> {
    let dept_eid = EntityId::from_str(&dept.dept_uuid)?;
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

// 获取当前部门的所有父级部门ID
pub async fn find_parents_dept_id(
    db: &DatabaseConnection,
    current_dept_id: i32
) -> Result<Vec<i32>, AppError> {
    let all_depts = departments::Entity::find()
        .select_only()
        .columns([departments::Column::DeptId, departments::Column::ParentId])
        .filter(departments::Column::IsDeleted.eq(false))
        .into_tuple::<(i32, i32)>()
        .all(db)
        .await?;

    let parent_map: HashMap<i32, i32> = all_depts.into_iter().collect();

    let mut parent_ids = Vec::new();
    let mut current_id = Some(current_dept_id);

    while let Some(id) = current_id {
        if id == ROOT_DEPARTMENT_ID { break; }
        if let Some(&parent_id) = parent_map.get(&id) {
            if parent_id != ROOT_DEPARTMENT_ID {
                parent_ids.push(parent_id);
            }
            current_id = Some(parent_id);
        } else {
            break;
        }
    }
    Ok(parent_ids)
}


#[async_recursion]
pub async fn build_dept_tree(
    depts_by_parent: &HashMap<i32, Vec<&departments::Model>>,
    id_to_uuid_map: &HashMap<i32, &String>,
    parent_id: i32,
) -> Result<Vec<DeptTreeNode>, AppError> {
    let mut tree_nodes = Vec::new();

    if let Some(children_models) = depts_by_parent.get(&parent_id) {
        for dept in children_models {
            // 递归构建子树
            let children = build_dept_tree(
                depts_by_parent,
                id_to_uuid_map,
                dept.dept_id,
            ).await?;

            let parent_uuid = id_to_uuid_map
                .get(&dept.parent_id)
                .map(|s| s.to_string())
                // 如果 parent_id 是 0 (根节点)，则使用虚拟的根 UUID
                .unwrap_or_else(|| ROOT_DEPARTMENT_UUID.to_string());

            tree_nodes.push(DeptTreeNode {
                uuid: dept.dept_uuid.clone(),
                name: dept.name.clone(),
                desc: dept.desc.clone(),
                order: dept.order,
                parent_uuid,
                children,
            });
        }
    }

    tree_nodes.sort_by_key(|n| n.order);
    Ok(tree_nodes)
}


pub async fn build_dept_tree_optimized_with_uuid(
    all_departments: &[departments::Model],
    root_parent_id: i32,
) -> Result<Vec<DeptTreeNode>, AppError> {
    if all_departments.is_empty() {
        return Ok(vec![]);
    }

    let depts_by_parent: HashMap<i32, Vec<&departments::Model>> =
        all_departments.iter().fold(HashMap::new(), |mut acc, dept| {
            acc.entry(dept.parent_id).or_default().push(dept);
            acc
        });

    let id_to_uuid_map: HashMap<i32, &String> = all_departments
        .iter()
        .map(|dept| (dept.dept_id, &dept.dept_uuid))
        .collect();

    let tree_node = build_dept_tree(
        &depts_by_parent,
        &id_to_uuid_map,
        root_parent_id,
    ).await?;

    Ok(tree_node)
}


// 获取当前用户所有的子部门id
pub async fn children_dept(
    db: &DatabaseConnection,
    parent_id: i32,
) -> Result<Vec<i32>, DbErr> {
    // 获取所有部门并按 order 排序
    let all_depts = departments::Entity::find()
        .columns([departments::Column::DeptId, departments::Column::ParentId])
        .order_by_asc(departments::Column::Order)
        .all(db)
        .await?;

    // 初始化结果集（包含自身）
    let mut dept_ids = vec![parent_id];
    // 使用 VecDeque 作为队列（比 Vec 更适合 FIFO 操作）
    let mut to_visit = VecDeque::from([parent_id]);

    while let Some(current_id) = to_visit.pop_front() {
        // 查找当前部门的所有直接子部门
        let children: Vec<i32> = all_depts
            .iter()
            .filter(|dept| dept.parent_id == current_id)
            .map(|dept| dept.dept_id)
            .collect();

        // 添加到结果集和待访问队列
        dept_ids.extend(&children);
        to_visit.extend(children);
    }

    Ok(dept_ids)
}

// 组装用户信息(用户信息、角色信息、部门信息)
pub async fn assemble_user_info(
    db: &DatabaseConnection,
    users_with_dept: Vec<(users::Model, Option<departments::Model>)>,
) -> Result<Vec<UserResponse>, AppError> {
    // 2. 收集所有用户ID
    let user_ids: Vec<i32> = users_with_dept
        .iter()
        .map(|(user, _)| user.user_id)
        .collect();

    // 3. 批量查询所有用户的用户组信息
    let user_group_relations = user_group_members::Entity::find()
        .find_also_related(user_groups::Entity)
        .filter(user_group_members::Column::UserId.is_in(user_ids))
        .all(db)
        .await?;

    // 4. 按用户ID分组用户组信息
    let mut user_groups_map: HashMap<i32, Vec<GroupResponse>> = HashMap::new();
    for (relation, group_opt) in user_group_relations {
        if let Some(group) = group_opt {
            user_groups_map
                .entry(relation.user_id)
                .or_insert_with(Vec::new)
                .push(GroupResponse {
                    uuid: group.user_group_uuid,
                    name: group.name,
                });
        }
    }

    // 5. 构建最终结果
    let result = users_with_dept
        .into_iter()
        .map(|(user, dept)| UserResponse {
            uuid: user.user_uuid,
            username: user.username.clone(),
            alias: user.alias.clone(),
            email: user.email.clone(),
            phone: user.phone.clone(),
            is_active: user.is_active,
            dept: dept.map(|d| DeptResponse {
                uuid: d.dept_uuid,
                name: d.name,
            }),
            groups: user_groups_map
                .get(&user.user_id)
                .cloned()
                .unwrap_or_default(),
            avatar: user.avatar.clone(),
            last_login: user.last_login,
        })
        .collect();
    Ok(result)
}
