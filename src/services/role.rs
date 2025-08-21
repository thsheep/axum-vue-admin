// 角色管理路由

use crate::config::state::AppState;
use crate::entity::{roles, user_roles};
use crate::errors::app_error::AppError;
use crate::schemas::auth::CurrentUser;
use crate::schemas::cedar_policy::CedarContext;
use crate::schemas::role::{
    CreateRoleDto, QueryParams, RoleFieldResponse, RoleResponse,
    UpdateRoleDto,
};
use crate::utils::cedar_utils::{entities2json, AuthAction, ResourceType};
use crate::{bad_request, conflict, not_found};
use cedar_policy::{Entities, Entity, EntityId, EntityTypeName, EntityUid, RestrictedExpression};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect, Select, Set, TransactionTrait};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use tracing::debug;

#[derive(Clone)]
pub struct RoleService {
    app_state: AppState,
}


impl RoleService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }
    
    pub async fn validate_role_id(&self, role_ids: &Vec<i32>) -> Result<(), AppError> {
        let existing_cont = roles::Entity::find()
            .filter(roles::Column::RoleId.is_in(role_ids.to_vec()))
            .count(&self.app_state.db)
            .await?;
        if existing_cont != role_ids.len() as u64 {
            return Err(bad_request!("Some Role do not exist".to_string()));
        }
        Ok(())
    }
    
    pub async fn get_role_entities(&self, role_ids: &Vec<i32>) -> Result<Entities, AppError> {
        let roles = roles::Entity::find()
            .filter(roles::Column::RoleId.is_in(role_ids.to_vec()))
            .all(&self.app_state.db)
            .await?;
        let mut entities = HashSet::new();
        for role in roles {
            let role_eid = EntityId::from_str(&role.role_id.to_string())?;
            let role_typename = EntityTypeName::from_str("Role")?;
            let role_e_uid = EntityUid::from_type_name_and_id(role_typename, role_eid);

            let mut attrs = HashMap::new();
            let name_expr = RestrictedExpression::new_string(role.role_name);
            attrs.insert("name".to_string(), name_expr);

            let parents = HashSet::new();
            let role_entity = Entity::new(role_e_uid, attrs, parents)?;
            entities.insert(role_entity);
        }
        let schema = self.app_state.auth_service.get_schema_copy().await;
        let entities = Entities::from_entities(entities, Some(&schema))?;
        let entities_json = entities2json(&entities)?;
        debug!("Role: {:?}; Entities Json: {}", role_ids, entities_json);
        Ok(entities)
    }

    pub async fn list_roles(&self,
                            current_user: CurrentUser,
                            context: CedarContext,
                            params: QueryParams) -> Result<(Vec<Value>, u64), AppError> {

        self.app_state
            .auth_service
            .check_permission(
                current_user,
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

        let es = self.get_role_entities(&vec![role_id]).await?;

        self.app_state
            .auth_service
            .check_permission_with_entities(
                current_user,
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
                current_user,
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

        let es = self.get_role_entities(&vec![role_id]).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                current_user,
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

        let es = self.get_role_entities(&vec![role_id]).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                current_user,
                context,
                AuthAction::DeleteRole,
                ResourceType::Role(Some(role_id)),
                es
            )
            .await?;

        // 删除角色时，需要同时删除 user_roles 表中与该角色关联的所有记录
        let txn = self.app_state.db.begin().await?;

        user_roles::Entity::delete_many()
            .filter(user_roles::Column::RoleId.eq(role_id))
            .exec(&txn)
            .await?;
        
        roles::Entity::delete_by_id(role_id).exec(&txn).await?;

        txn.commit().await?;
        Ok(())
    }
}
