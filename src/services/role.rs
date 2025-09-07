// 角色管理路由

use crate::config::state::AppState;
use crate::entity::{group_roles, roles, user_group_members, user_roles};
use crate::errors::app_error::AppError;
use crate::schemas::auth::CurrentUser;
use crate::schemas::cedar_policy::CedarContext;
use crate::schemas::role::{
    CreateRoleDto, QueryParams, RoleFieldResponse, RoleResponse,
    UpdateRoleDto,
};
use crate::utils::cedar_utils::{entities2json, AuthAction, ResourceType, ENTITY_TYPE_ROLE, ENTITY_ATTR_NAME};
use crate::{bad_request, conflict, not_found};
use cedar_policy::{Entities, Entity, EntityId, EntityTypeName, EntityUid, RestrictedExpression, Schema};
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect, QueryTrait, Select, Set, TransactionTrait};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use tracing::debug;
use crate::schemas::user::UserID;

#[derive(Clone)]
pub struct RoleService {
    app_state: AppState,
}


impl RoleService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }


    pub async fn list_roles(&self,
                            current_user: CurrentUser,
                            context: CedarContext,
                            params: QueryParams) -> Result<(Vec<Value>, u64), AppError> {

        self.app_state
            .auth_service
            .check_permission(
                current_user.user_id,
                context,
                AuthAction::ViewRole,
                ResourceType::Role(None),
            )
            .await?;

        // 构建基础查询
        let mut base_query = roles::Entity::find();

        // 应用通用过滤条件
        if let Some(role_name) = &params.name {
            base_query = base_query.filter(roles::Column::RoleName.contains(role_name));
        }

        // 根据 fields 参数决定查询策略
        match params.fields.as_ref() {
            Some(fields) => {
                self.list_roles_with_fields(base_query, fields, &params).await
            }
            None => {
                self.list_roles_full(base_query, &params).await
            }
        }
    }


    // 处理指定字段查询
    async fn list_roles_with_fields(
        &self,
        base_query: Select<roles::Entity>,
        fields: &str,
        params: &QueryParams,
    ) -> Result<(Vec<Value>, u64), AppError> {
        let requested_fields: Vec<&str> = fields.split(',').map(|s| s.trim()).collect();
        let mut select = base_query.select_only();

        // 动态添加字段
        for field in &requested_fields {
            select = match *field {
                "id" => select.column_as(roles::Column::RoleId, "id"),
                "name" => select.column_as(roles::Column::RoleName, "name"),
                "description" => select.column_as(roles::Column::Description, "description"),
                "created_at" => select.column_as(roles::Column::CreatedAt, "created_at"),
                _ => select, // 忽略未知字段
            };
        }

        let paginator = select
            .into_model::<RoleFieldResponse>()
            .paginate(&self.app_state.db, params.page_size);

        let total = paginator.num_items().await?;
        let results = paginator.fetch_page(params.page - 1).await?;

        // 安全的序列化
        let response = results
            .into_iter()
            .map(|role| serde_json::to_value(role))
            .collect::<Result<Vec<_>, _>>()?;

        Ok((response, total))
    }

    // 处理完整查询
    async fn list_roles_full(
        &self,
        base_query: Select<roles::Entity>,
        params: &QueryParams,
    ) -> Result<(Vec<Value>, u64), AppError> {
        let paginator = base_query.paginate(&self.app_state.db, params.page_size);
        let total = paginator.num_items().await?;
        let results = paginator.fetch_page(params.page - 1).await?;

        let response = results
            .into_iter()
            .map(|role| serde_json::to_value(RoleResponse::from(role)))
            .collect::<Result<Vec<_>, _>>()?;

        Ok((response, total))
    }



    pub async fn get_role(&self,
                          current_user: CurrentUser,
                          context: CedarContext,
                          role_id: i32) -> Result<RoleResponse, AppError> {

        let schema = self.app_state.auth_service.get_schema_copy().await;
        let es = get_role_entities(&self.app_state.db, &vec![role_id], &schema).await?;

        self.app_state
            .auth_service
            .check_permission_with_entities(
                current_user.user_id,
                context,
                AuthAction::ViewRole,
                ResourceType::Role(Some(role_id)),
                es
            )
            .await?;

        let role = roles::Entity::find_by_id(role_id)
            .column_as(roles::Column::RoleId, "id")
            .column_as(roles::Column::RoleName, "name")
            .column(roles::Column::Description)
            .column(roles::Column::CreatedAt)
            .into_model::<RoleResponse>()
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!("Role Not Found".to_string()))?;
        Ok(role)
    }

    pub async fn create_role(&self,
                             current_user: CurrentUser,
                             context: CedarContext,
                             dto: CreateRoleDto) -> Result<RoleResponse, AppError> {

        self.app_state
            .auth_service
            .check_permission(
                current_user.user_id,
                context,
                AuthAction::CreateRole,
                ResourceType::Role(None),
            )
            .await?;

        if roles::Entity::find()
            .filter(roles::Column::RoleName.eq(&dto.name))
            .one(&self.app_state.db)
            .await?
            .is_some()
        {
            return Err(conflict!("Role already exists".to_string()));
        }

        let role = roles::ActiveModel {
            role_name: Set(dto.name),
            description: Set(Some(dto.description)),
            ..Default::default()
        };

        let role = role.insert(&self.app_state.db).await?;
        Ok(RoleResponse::from(role))
    }

    pub async fn update_role(&self,
                             current_user: CurrentUser,
                             context: CedarContext,
                             role_id: i32,
                             dto: UpdateRoleDto) -> Result<RoleResponse, AppError> {

        let schema = self.app_state.auth_service.get_schema_copy().await;
        let es = get_role_entities(&self.app_state.db, &vec![role_id], &schema).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                current_user.user_id,
                context,
                AuthAction::UpdateRole,
                ResourceType::Role(Some(role_id)),
                es
            )
            .await?;


        let mut role: roles::ActiveModel = roles::Entity::find_by_id(role_id)
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!("Role Not Found".to_string()))?
            .into();

        if let Some(name) = dto.name {
            role.role_name = Set(name);
        }

        if let Some(description) = dto.description {
            role.description = Set(Some(description));
        }
        
        let role = role.update(&self.app_state.db).await?;
        Ok(RoleResponse::from(role))
    }

    pub async fn delete_role(&self,
                             current_user: CurrentUser,
                             context: CedarContext,
                             role_id: i32) -> Result<(), AppError> {

        let schema = self.app_state.auth_service.get_schema_copy().await;
        let es = get_role_entities(&self.app_state.db, &vec![role_id], &schema).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                current_user.user_id,
                context,
                AuthAction::DeleteRole,
                ResourceType::Role(Some(role_id)),
                es
            )
            .await?;


        // 删除角色时，需要同时删除 user_roles 表中与该角色关联的所有记录
        let txn = self.app_state.db.begin().await?;

        roles::Entity::delete_by_id(role_id).exec(&txn).await?;


        user_roles::Entity::delete_many()
            .filter(user_roles::Column::RoleId.eq(role_id))
            .exec(&txn)
            .await?;

        group_roles::Entity::delete_many()
            .filter(group_roles::Column::RoleId.eq(role_id))
            .exec(&txn)
            .await?;

        txn.commit().await?;
        Ok(())
    }
}


pub async fn get_role_models_by_user_id(db: &DatabaseConnection, user_id: UserID) -> Result<Vec<roles::Model>, AppError> {
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
        .all(db)
        .await?;
    Ok(all_roles)
}


pub async fn get_role_entities(db: &DatabaseConnection, role_ids: &Vec<i32>, schema: &Schema) -> Result<Entities, AppError> {
    let roles = roles::Entity::find()
        .filter(roles::Column::RoleId.is_in(role_ids.to_vec()))
        .all(db)
        .await?;
    let mut entities = HashSet::new();
    for role in roles {
        let role_eid = EntityId::from_str(&role.role_id.to_string())?;
        let role_typename = EntityTypeName::from_str(ENTITY_TYPE_ROLE)?;
        let role_e_uid = EntityUid::from_type_name_and_id(role_typename, role_eid);

        let mut attrs = HashMap::new();
        let name_expr = RestrictedExpression::new_string(role.role_name);
        attrs.insert(ENTITY_ATTR_NAME.to_string(), name_expr);

        let parents = HashSet::new();
        let role_entity = Entity::new(role_e_uid, attrs, parents)?;
        entities.insert(role_entity);
    }
    let entities = Entities::from_entities(entities, Some(&schema))?;
    let entities_json = entities2json(&entities)?;
    debug!("Role: {:?}; Entities Json: {}", role_ids, entities_json);
    Ok(entities)
}
