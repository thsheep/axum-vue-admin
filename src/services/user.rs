use crate::config::state::AppState;
use crate::entity::{
    departments, group_roles, roles, user_group_members, user_groups, user_roles, users,
};
use crate::errors::app_error::AppError;
use crate::schemas::auth::CurrentUser;
use crate::schemas::cedar_policy::CedarContext;
use crate::schemas::user::{
    AssignRoleDto, CreateUserDto, DeptResponse, DirectRole, GroupResponse, GroupRole,
    QueryParams, UpdateUserDto, UserResponse, UserRoleInfo
};
use crate::services::department::DepartmentService;
use crate::services::groups::GroupService;
use crate::services::role::RoleService;
use crate::utils::cedar_utils::{entities2json, AuthAction, ResourceType};
use crate::utils::crypto::hash_password;
use crate::{conflict, not_found};
use cedar_policy::{Entities, Entity, EntityId, EntityTypeName, EntityUid, RestrictedExpression};
use sea_orm::JoinType::InnerJoin;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, JoinType, ModelTrait,
    PaginatorTrait, QueryFilter, QuerySelect, QueryTrait, RelationTrait, SelectColumns, Set,
    TransactionTrait,
};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use tracing::debug;

const ENTITY_TYPE_USER: &str = "User";
const ENTITY_TYPE_GROUP: &str = "Group";
const ENTITY_TYPE_ROLE: &str = "Role";
const ENTITY_ATTR_NAME: &str = "name";
const ROLE_SOURCE_DIRECT: &str = "direct";
const ROLE_SOURCE_GROUP: &str = "group";

#[derive(Clone)]
pub struct UserService {
    app_state: AppState,
    group_service: GroupService,
    department_service: DepartmentService,
    role_service: RoleService,
}

impl UserService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            app_state: app_state.clone(),
            group_service: GroupService::new(app_state.clone()),
            department_service: DepartmentService::new(app_state.clone()),
            role_service: RoleService::new(app_state.clone()),
        }
    }
    // 获取用户实体信息
    pub async fn get_user_entities(&self, user_id: i32) -> Result<Entities, AppError> {
        // 获取 user_id 的用户名
        let user = users::Entity::find_by_id(user_id)
            .select_column(users::Column::Username)
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!("User {} not found", user_id))?;


        let all_roles = self.get_user_role_models(user_id).await?;

        if all_roles.is_empty() {
            debug!("User {} has no roles assigned", user_id);
        }

        // 用户所属组
        let groups = user_groups::Entity::find()
            .select_column(user_groups::Column::Name)
            .join(InnerJoin, user_groups::Relation::UserGroupMembers.def())
            .filter(user_group_members::Column::UserId.eq(user_id))
            .all(&self.app_state.db)
            .await?;

        // 用户所属部门
        let department = departments::Entity::find()
            .select_column(departments::Column::Name)
            .join(InnerJoin, departments::Relation::Users.def())
            .filter(users::Column::UserId.eq(user_id))
            .one(&self.app_state.db)
            .await?;

        // 转换为 CedarEntity
        let mut entities = HashSet::new();
        let mut user_parent_uids = HashSet::new();

        if department.is_some() {
            let department = department.unwrap();
            // 用户所有的子部门
            let child_dept_entities = self.department_service.find_descendants_entities(department.dept_id).await?;
            for child_dept_entity in child_dept_entities {
                let dept_e_uid = child_dept_entity.uid();
                user_parent_uids.insert(dept_e_uid);
                entities.insert(child_dept_entity);
            }
        };

        for group in groups {
            let group_eid = EntityId::from_str(group.user_group_id.to_string().as_str())?;
            let group_type_name = EntityTypeName::from_str(ENTITY_TYPE_GROUP)?;
            let group_e_uid = EntityUid::from_type_name_and_id(group_type_name, group_eid);

            let mut attrs = HashMap::new();
            let name_expr = RestrictedExpression::new_string(group.name);
            attrs.insert(ENTITY_ATTR_NAME.to_string(), name_expr);

            let parents = HashSet::new();
            let group_entity = Entity::new(group_e_uid.clone(), attrs, parents)?;
            entities.insert(group_entity);
            user_parent_uids.insert(group_e_uid);
        };

        for role in all_roles {
            let role_eid = EntityId::from_str(role.role_id.to_string().as_str())?;
            let role_type_name = EntityTypeName::from_str(ENTITY_TYPE_ROLE)?;
            let role_e_uid = EntityUid::from_type_name_and_id(role_type_name, role_eid);

            let mut attrs = HashMap::new();
            let name_expr = RestrictedExpression::new_string(role.role_name);
            attrs.insert(ENTITY_ATTR_NAME.to_string(), name_expr);

            let parents = HashSet::new();
            let role_entity = Entity::new(role_e_uid.clone(), attrs, parents)?;
            entities.insert(role_entity);
            user_parent_uids.insert(role_e_uid);

        }

        let user_eid = EntityId::from_str(user.user_id.to_string().as_str())?;
        let user_type_name = EntityTypeName::from_str(ENTITY_TYPE_USER)?;
        let user_e_uid = EntityUid::from_type_name_and_id(user_type_name, user_eid);
        let mut attrs = HashMap::new();

        let name_expr = RestrictedExpression::new_string(user.username);
        attrs.insert(ENTITY_ATTR_NAME.to_string(), name_expr);
        let user_entity = Entity::new(user_e_uid, attrs, user_parent_uids)?;
        entities.insert(user_entity);

        let schema = self.app_state.auth_service.get_schema_copy().await;
        let verified_entities = Entities::from_entities(entities, Some(&schema))?;
        let entities_json = entities2json(&verified_entities)?;
        debug!("User:{}; Entities Json: {}", user_id, entities_json);
        Ok(verified_entities)
    }

    pub async fn get_user_role_models(&self, user_id: i32) -> Result<Vec<roles::Model>, AppError> {
        // --- 子查询 1: 获取直接分配给用户的角色 ID ---
        let direct_role_ids_query = user_roles::Entity::find()
            .select_only() // 只选择特定列
            .column(user_roles::Column::RoleId) // 我们只需要 role_id
            .filter(user_roles::Column::UserId.eq(user_id));

        // --- 子查询 2: 获取通过用户组继承的角色 ID ---
        // 首先，找到该用户所属的所有 group_id
        let group_ids_query = user_group_members::Entity::find()
            .select_only()
            .column(user_group_members::Column::GroupId)
            .filter(user_group_members::Column::UserId.eq(user_id));

        // 然后，基于上面的 group_id 找到所有关联的 role_id
        let group_role_ids_query = group_roles::Entity::find()
            .select_only()
            .column(group_roles::Column::RoleId)
            .filter(group_roles::Column::GroupId.in_subquery(group_ids_query.into_query()));

        // --- 主查询: 获取所有符合条件的角色信息 ---
        // 使用 Condition::any() (即 OR) 来合并两个子查询的结果
        let all_roles = roles::Entity::find()
            .filter(
                Condition::any()
                    // 条件1: role_id 在直接分配的角色 ID 列表中
                    .add(roles::Column::RoleId.in_subquery(direct_role_ids_query.into_query()))
                    // 条件2: role_id 在通过用户组继承的角色 ID 列表中
                    .add(roles::Column::RoleId.in_subquery(group_role_ids_query.into_query())),
            )
            .all(&self.app_state.db)
            .await?;
        Ok(all_roles)
    }
    

    pub async fn list_users(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        params: QueryParams,
    ) -> Result<(Vec<UserResponse>, u64), AppError> {
        self.app_state
            .auth_service
            .check_permission(
                current_user,
                context,
                AuthAction::ViewUser,
                ResourceType::User(None),
            )
            .await?;

        let mut query = users::Entity::find().filter(users::Column::IsActive.eq(true));
        if let Some(username) = params.username {
            query = query.filter(users::Column::Username.contains(username));
        }

        if let Some(email) = params.email {
            query = query.filter(users::Column::Email.contains(email));
        }

        if let Some(dept_id) = params.dept_id {
            query = query.filter(users::Column::DeptId.eq(dept_id));
        }

        // 1. 查询所有用户及其部门信息
        let paginate = query
            .find_also_related(departments::Entity)
            .paginate(&self.app_state.db, params.page_size);

        let total = paginate.num_items().await?;

        let users_with_dept = paginate.fetch_page(params.page - 1).await?;

        // 2. 收集所有用户ID
        let user_ids: Vec<i32> = users_with_dept
            .iter()
            .map(|(user, _)| user.user_id)
            .collect();

        // 3. 批量查询所有用户的用户组信息
        let user_group_relations = user_group_members::Entity::find()
            .find_also_related(user_groups::Entity)
            .filter(user_group_members::Column::UserId.is_in(user_ids))
            .all(&self.app_state.db)
            .await?;

        // 4. 按用户ID分组用户组信息
        let mut user_groups_map: HashMap<i32, Vec<GroupResponse>> = HashMap::new();
        for (relation, group_opt) in user_group_relations {
            if let Some(group) = group_opt {
                user_groups_map
                    .entry(relation.user_id)
                    .or_insert_with(Vec::new)
                    .push(GroupResponse {
                        id: group.user_group_id,
                        name: group.name,
                    });
            }
        }

        // 5. 构建最终结果
        let result = users_with_dept
            .into_iter()
            .map(|(user, dept)| UserResponse {
                id: user.user_id,
                username: user.username.clone(),
                alias: user.alias.clone(),
                email: user.email.clone(),
                phone: user.phone.clone(),
                is_active: user.is_active,
                dept: dept.map(|d| DeptResponse {
                    id: d.dept_id,
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

        Ok((result, total))
    }

    pub async fn get_user(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        id: i32,
    ) -> Result<UserResponse, AppError> {
        let es = self.get_user_entities(id).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                current_user,
                context,
                AuthAction::ViewUser,
                ResourceType::User(Some(id)),
                es,
            )
            .await?;

        // 查询用户及其部门信息
        let user_with_dept = users::Entity::find_by_id(id)
            .find_also_related(departments::Entity)
            .one(&self.app_state.db)
            .await?;

        if let Some((user, dept)) = user_with_dept {
            // 查询该用户的所有用户组
            let user_groups = user_groups::Entity::find()
                .join(
                    InnerJoin,
                    user_groups::Relation::UserGroupMembers.def(),
                )
                .filter(user_group_members::Column::UserId.eq(user.user_id))
                .all(&self.app_state.db)
                .await?;

            let user_response = UserResponse {
                id: user.user_id,
                username: user.username,
                alias: user.alias,
                email: user.email,
                phone: user.phone,
                is_active: user.is_active,
                dept: dept.map(|d| DeptResponse {
                    id: d.dept_id,
                    name: d.name,
                }),
                groups: user_groups
                    .into_iter()
                    .map(|g| GroupResponse {
                        id: g.user_group_id,
                        name: g.name,
                    })
                    .collect(),
                avatar: user.avatar,
                last_login: user.last_login,
            };

            Ok(user_response)
        } else {
            Err(not_found!("User not found".to_string()))
        }
    }

    pub async fn create_user(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        dto: CreateUserDto,
    ) -> Result<UserResponse, AppError> {
        self.department_service.validate_dept_id(dto.dept).await?;
        self.group_service.validate_group_ids(&dto.groups).await?;

        let dept_es = self.department_service.get_dept_entities(dto.dept).await?;
        let group_es = self.group_service.get_group_entities(&dto.groups).await?;
        let merged_es = dept_es.add_entities(group_es.clone(), None)?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                current_user.clone(),
                context.clone(),
                AuthAction::CreateUser,
                ResourceType::User(None),
                merged_es.clone(),
            )
            .await?;

        for group_id in &dto.groups {
            self.app_state
                .auth_service
                .check_permission_with_entities(
                    current_user.clone(),
                    context.clone(),
                    AuthAction::CreateUser,
                    ResourceType::Group(Some(*group_id)),
                    group_es.clone(),
                ).await?;
        }

        let txn = self.app_state.db.begin().await?;

        if users::Entity::find()
            .filter(
                users::Column::Username
                    .eq(&dto.username)
                    .or(users::Column::Email.eq(&dto.email)),
            )
            .one(&txn)
            .await?
            .is_some()
        {
            return Err(conflict!("Username or email already exists".to_string(),));
        }

        let hashed_password = hash_password(&dto.password)?;

        let user = users::ActiveModel {
            username: Set(dto.username),
            email: Set(dto.email),
            password: Set(hashed_password),
            dept_id: Set(Some(dto.dept)),
            alias: Set(dto.alias),
            phone: Set(dto.phone),
            is_active: Set(dto.is_active),
            ..Default::default()
        };

        let user = user.insert(&txn).await?;

        let user_groups = dto
            .groups
            .into_iter()
            .map(|group_id| user_group_members::ActiveModel {
                user_id: Set(user.user_id),
                group_id: Set(group_id),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        user_group_members::Entity::insert_many(user_groups)
            .exec(&txn)
            .await?;

        txn.commit().await?;

        Ok(UserResponse::from(user))
    }

    pub async fn update_user(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        user_id: i32,
        dto: UpdateUserDto) -> Result<UserResponse, AppError> {

        let update_user_es = self.get_user_entities(user_id).await?;
        let es = self.group_service.get_group_entities(&dto.groups).await?;
        let merged_es = update_user_es.add_entities(es, None)?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                current_user,
                context,
                AuthAction::UpdateUser,
                ResourceType::User(Some(user_id)),
                merged_es,
            )
            .await?;

        let txn = self.app_state.db.begin().await?;

        let mut user: users::ActiveModel = users::Entity::find_by_id(user_id)
            .one(&txn)
            .await?
            .ok_or(not_found!("User Not found".to_string()))?
            .into();

        user.email = Set(dto.email);

        user.username = Set(dto.username);

        user.dept_id = Set(Some(dto.dept));

        if let Some(alias) = dto.alias {
            user.alias = Set(Some(alias));
        }

        if let Some(phone) = dto.phone {
            user.phone = Set(Some(phone));
        }

        if let Some(is_active) = dto.is_active {
            user.is_active = Set(is_active);
        }

        let user = user.update(&txn).await?;

        user_group_members::Entity::delete_many()
            .filter(user_group_members::Column::UserId.eq(user_id))
            .exec(&txn)
            .await?;

        user_group_members::Entity::insert_many(dto.groups.into_iter().map(|group_id| {
            user_group_members::ActiveModel {
                user_id: Set(user.user_id),
                group_id: Set(group_id),
                ..Default::default()
            }
        }))
        .exec(&txn)
        .await?;

        txn.commit().await?;

        Ok(UserResponse::from(user))
    }

    pub async fn delete_user(&self,
                             current_user: CurrentUser,
                             context: CedarContext,
                             user_id: i32) -> Result<(), AppError> {
        let delete_user_es = self.get_user_entities(user_id).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                current_user,
                context,
                AuthAction::DeleteUser,
                ResourceType::User(Some(user_id)),
                delete_user_es,
            ).await?;

        // 用户不删除 只是禁用
        let user = users::ActiveModel {
            user_id: Set(user_id),
            is_active: Set(false),
            ..Default::default()
        };
        user.update(&self.app_state.db).await?;

        Ok(())
    }

    pub async fn user_roles(&self,
                            current_user: CurrentUser,
                            context: CedarContext,
                            user_id: i32) -> Result<Vec<UserRoleInfo>, AppError> {

        self.app_state
            .auth_service
            .check_permission(
                current_user,
                context,
                AuthAction::ViewRole,
                ResourceType::User(None),
            )
            .await?;

        let mut all_roles = Vec::new();

        // 获取直接角色
        let direct_roles = self.get_user_direct_roles(user_id).await?;
        for direct_role in direct_roles {
            all_roles.push(UserRoleInfo {
                id: Some(direct_role.id),
                role_name: direct_role.role_name,
                source: ROLE_SOURCE_DIRECT.to_string(),
                group_name: None,
            });
        }

        // 获取组角色
        let group_roles = self.get_user_group_roles(user_id).await?;
        for group_role in group_roles {
            all_roles.push(UserRoleInfo {
                id: Some(group_role.id),
                role_name: group_role.role_name,
                source: ROLE_SOURCE_GROUP.to_string(),
                group_name: Some(group_role.group_name),
            });
        }

        Ok(all_roles)
    }

    // 获取用户直接分配的角色
    async fn get_user_direct_roles(&self, user_id: i32) -> Result<Vec<DirectRole>, AppError> {
        let roles = roles::Entity::find()
            .select_only()
            .column_as(roles::Column::RoleId, "id")
            .column(roles::Column::RoleName)
            .join(JoinType::InnerJoin, roles::Relation::UserRoles.def())
            .filter(user_roles::Column::UserId.eq(user_id))
            .into_tuple::<(i32, String)>()
            .all(&self.app_state.db)
            .await?
            .into_iter()
            .map(|(id, name)| DirectRole {
                id,
                role_name: name,
            })
            .collect();

        Ok(roles)
    }

    // 获取用户通过组获得的角色（包含组信息）
    async fn get_user_group_roles(&self, user_id: i32) -> Result<Vec<GroupRole>, AppError> {
        let roles = roles::Entity::find()
            .select_only()
            .columns([roles::Column::RoleId, roles::Column::RoleName])
            .column_as(user_groups::Column::Name, "group_name")
            .join(InnerJoin, roles::Relation::GroupRoles.def())
            .join(InnerJoin, group_roles::Relation::UserGroups.def())
            .join(
                InnerJoin,
                user_groups::Relation::UserGroupMembers.def(),
            )
            .filter(user_group_members::Column::UserId.eq(user_id))
            .into_tuple::<(i32, String, String)>()
            .all(&self.app_state.db)
            .await?;

        let group_roles = roles
            .into_iter()
            .map(|(role_id, role_name, group_name)| GroupRole {
                id: role_id,
                role_name,
                group_name,
            })
            .collect();

        Ok(group_roles)
    }

    pub async fn assign_roles(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        user_id: i32,
        dto: AssignRoleDto,
    ) -> Result<(), AppError> {

        self.role_service.validate_role_id(&dto.ids).await?;
        let role_es = self.role_service.get_role_entities(&dto.ids).await?;

        let user_es = self.get_user_entities(user_id).await?;

        let merged_es = role_es.add_entities(user_es, None)?;

        self.app_state.auth_service
            .check_permission_with_entities(
                current_user,
                context,
                AuthAction::AssignRole,
                ResourceType::Role(Some(user_id)),
                merged_es,
            ).await?;

        let txn = self.app_state.db.begin().await?;
        // 删除用户原有的角色
        user_roles::Entity::delete_many()
            .filter(user_roles::Column::UserId.eq(user_id))
            .exec(&txn)
            .await?;
        // 给用户分配角色
        user_roles::Entity::insert_many(dto.ids.into_iter().map(|role_id| {
            user_roles::ActiveModel {
                user_id: Set(user_id),
                role_id: Set(role_id),
                ..Default::default()
            }
        }))
        .exec(&txn)
        .await?;
        txn.commit().await?;

        Ok(())
    }

    pub async fn revoke_roles(&self,
                              current_user: CurrentUser,
                              context: CedarContext,
                              user_id: i32,
                              role_id: i32) -> Result<(), AppError> {
        let role_ids = vec![role_id];
        self.role_service.validate_role_id(&role_ids).await?;

        let role_es = self.role_service.get_role_entities(&role_ids).await?;
        let user_es = self.get_user_entities(user_id).await?;
        let merged_es = role_es.add_entities(user_es, None)?;

        self.app_state.auth_service
            .check_permission_with_entities(
                current_user,
                context,
                AuthAction::RevokeRole,
                ResourceType::User(Some(user_id)),
                merged_es,
            ).await?;

        // 移除用户的角色
        user_roles::Entity::delete_many()
            .filter(user_roles::Column::UserId.eq(user_id))
            .filter(user_roles::Column::RoleId.eq(role_id))
            .exec(&self.app_state.db)
            .await?;

        Ok(())
    }

}
