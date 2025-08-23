use std::any::Any;
use crate::config::state::AppState;
use crate::entity::{roles, user_groups, user_group_members, group_roles};

use crate::services::role::RoleService;
use crate::errors::app_error::AppError;
use crate::schemas::auth::CurrentUser;
use crate::schemas::cedar_policy::CedarContext;
use crate::schemas::groups::AssignRolesDto;
use crate::schemas::groups::{AssignUsersDto, CreateGroupDto, GroupResponse, GroupRoleResponse, QueryParams};
use crate::utils::cedar_utils::{entities2json, AuthAction, ResourceType, ENTITY_TYPE_GROUP, ENTITY_ATTR_NAME};
use crate::{bad_request, conflict, not_found};
use cedar_policy::{Entities, Entity, EntityId, EntityTypeName, EntityUid, RestrictedExpression};
// 用户组
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityOrSelect, EntityTrait, JoinType, ModelTrait, PaginatorTrait, QueryFilter, QuerySelect, RelationTrait, Select, Set, TransactionTrait};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use tracing::debug;

#[derive(Clone)]
pub struct GroupService {
    app_state: AppState,
    role_service: RoleService
}


impl GroupService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            app_state: app_state.clone(),
            role_service: RoleService::new(app_state)
        }
    }


    // 获取用户组实体信息
    pub async fn get_group_entities(&self, group_ids: &[i32]) -> Result<Entities, AppError> {
        let groups = user_groups::Entity::find()
            .column(user_groups::Column::Name)
            .filter(user_groups::Column::UserGroupId.is_in(group_ids.to_vec()))
            .all(&self.app_state.db)
            .await?;

        let mut entities = HashSet::new();
        for group in groups {
            let group_eid = EntityId::from_str(&group.user_group_id.to_string())?;
            let group_typename = EntityTypeName::from_str(ENTITY_TYPE_GROUP)?;
            let group_e_uid = EntityUid::from_type_name_and_id(group_typename, group_eid);
            
            let mut attrs = HashMap::new();
            let name_expr = RestrictedExpression::new_string(group.name);
            attrs.insert(ENTITY_ATTR_NAME.to_string(), name_expr);
            
            let parents = HashSet::new();
            let group_entity = Entity::new(group_e_uid, attrs, parents)?;
            entities.insert(group_entity);
        }

        let schema = self.app_state.auth_service.get_schema_copy().await;
        let verified_entities = Entities::from_entities(entities, Some(&schema))?;
        let entities_json = entities2json(&verified_entities)?;
        debug!("Groups:{:?}; Entities Json: {}", group_ids, entities_json);
        Ok(verified_entities)
    }

    // 验证用户组ID是否存在
    pub async fn validate_group_ids(&self, group_ids: &[i32]) -> Result<(), AppError> {
        let existing_count = user_groups::Entity::find()
            .filter(user_groups::Column::UserGroupId.is_in(group_ids.to_vec()))
            .count(&self.app_state.db)
            .await?;

        if existing_count != group_ids.len() as u64 {
            return Err(bad_request!("Some group do not exist".to_string()));
        }
        Ok(())
    }

    pub async fn list_groups(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        params: QueryParams,
    ) -> Result<(Vec<Value>, u64), AppError> {

        self.app_state
            .auth_service
            .check_permission(
                current_user,
                context,
                AuthAction::ViewGroup,
                ResourceType::Group(None),
            )
            .await?;
        
        let mut query = user_groups::Entity::find();
        if let Some(name) = &params.name {
            query = query.filter(user_groups::Column::Name.contains(name));
        }

        match params.fields.as_ref() {
            Some(fields) => {
                self.list_groups_with_fields(query, fields, &params).await
            }
            None => {
                self.list_groups_full(query, &params).await
            }
        }
    }

    // 指定字段查询
    async fn list_groups_with_fields(
        &self,
        query: Select<user_groups::Entity>,
        fields: &str,
        params: &QueryParams,
    ) -> Result<(Vec<Value>, u64), AppError> {
        let requested_fields: Vec<&str> = fields.split(',').map(|s| s.trim()).collect();
        let mut select = query.select_only();
        for field in &requested_fields {
            select = match *field {
                "id" => select.column_as(user_groups::Column::UserGroupId, "id"),
                "name" => select.column_as(user_groups::Column::Name, "name"),
                "description" => select.column_as(user_groups::Column::Description, "description"),
                "created_at" => select.column_as(user_groups::Column::CreatedAt, "created_at"),
                "updated_at" => select.column_as(user_groups::Column::UpdatedAt, "updated_at"),
                _ => select
            }
        }
        let paginator = select
            .into_model::<GroupResponse>()
            .paginate(&self.app_state.db, params.page_size);

        let total = paginator.num_items().await?;
        let results = paginator
            .fetch_page(params.page - 1).await?
            .into_iter()
            .map(| group | serde_json::to_value(group))
            .collect::<Result<Vec<_>, _>>()?;

        Ok((results, total))
    }

    // 完整查询
    async fn list_groups_full(
        &self,
        query: Select<user_groups::Entity>,
        params: &QueryParams,
    ) -> Result<(Vec<Value>, u64), AppError>{
        let paginator = query
            .paginate(&self.app_state.db, params.page_size);
        let total = paginator.num_items().await?;
        let results = paginator
            .fetch_page(params.page - 1)
            .await?
            .into_iter()
            .map(| group | serde_json::to_value(GroupResponse::from(group)))
            .collect::<Result<Vec<_>, _>>()?;

        Ok((results, total))
    }

    pub async fn create_group(&self,
                              current_user: CurrentUser,
                              context: CedarContext,
                              create_group_dto: CreateGroupDto) -> Result<GroupResponse, AppError> {

        self.app_state
            .auth_service
            .check_permission(
                current_user,
                context,
                AuthAction::CreateGroup,
                ResourceType::Group(None),
            )
            .await?;
        
        let txn = self.app_state.db.begin().await?;
        
        if user_groups::Entity::find()
            .filter(user_groups::Column::Name.eq(&create_group_dto.name))
            .one(&txn)
            .await?
            .is_some()
        {
            return Err(conflict!("group name already exists".to_string()));

        };
        
        let insert_model = user_groups::ActiveModel {
            name: Set(create_group_dto.name),
            description: Set(create_group_dto.description),
            ..Default::default()
        };

        let user_group = insert_model.insert(&txn).await?;
        
        txn.commit().await?;
        Ok(GroupResponse::from(user_group))
    }

    pub async fn get_group(&self,
                           current_user: CurrentUser,
                           context: CedarContext,
                           group_id: i32) -> Result<GroupResponse, AppError> {
        
        let es = self.get_group_entities(&vec![group_id]).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                current_user,
                context,
                AuthAction::ViewGroup,
                ResourceType::Group(Some(group_id)),
                es
            )
            .await?;
        
        let group = user_groups::Entity::find_by_id(group_id)
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!("group not found".to_string()))?;

        Ok(GroupResponse::from(group))
    }

    pub async fn update_group(&self,
                              current_user: CurrentUser,
                              context: CedarContext,
                              group_id: i32, 
                              update_group_dto: CreateGroupDto) -> Result<GroupResponse, AppError> {
        
        let es = self.get_group_entities(&vec![group_id]).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                current_user,
                context,
                AuthAction::UpdateGroup,
                ResourceType::Group(Some(group_id)),
                es
            )
            .await?;
        
        
        let mut group: user_groups::ActiveModel = user_groups::Entity::find_by_id(group_id)
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!("group not found".to_string()))?
            .into();

        group.name = Set(update_group_dto.name);
        group.description = Set(update_group_dto.description);

        let group = group.update(&self.app_state.db).await?;

        Ok(GroupResponse::from(group))
    }

    pub async fn delete_group(&self,
                              current_user: CurrentUser,
                              context: CedarContext,
                              group_id: i32) -> Result<(), AppError> {
        
        let es = self.get_group_entities(&vec![group_id]).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                current_user,
                context,
                AuthAction::DeleteGroup,
                ResourceType::Group(Some(group_id)),
                es
            )
            .await?;

        let txn = self.app_state.db.begin().await?;

        // 检查该组是否还有用户
        let user_count = user_group_members::Entity::find()
            .filter(user_group_members::Column::GroupId.eq(group_id))
            .count(&txn)
            .await?;

        if user_count > 0 {
            return Err(bad_request!("Group exists user"));
        }
        
        user_groups::Entity::delete_many()
            .filter(user_groups::Column::UserGroupId.eq(group_id))
            .exec(&txn)
            .await?;

        group_roles::Entity::delete_many()
            .filter(group_roles::Column::GroupId.eq(group_id))
            .exec(&txn)
            .await?;

        txn.commit().await?;
        Ok(())
    }

    pub async fn assign_users(&self, id: i32, add_users_dto: AssignUsersDto) -> Result<(), AppError> {
        
        let txn = self.app_state.db.begin().await?;
        if user_groups::Entity::find_by_id(id)
            .one(&txn)
            .await?
            .is_none()
        {
            return Err(not_found!("group not found".to_string()))
        };
        
        user_group_members::Entity::delete_many()
            .filter(user_group_members::Column::GroupId.eq(id))
            .exec(&txn).await?;
        

        user_group_members::Entity::insert_many( add_users_dto.user_ids.into_iter().map(|user_id| {
            user_group_members::ActiveModel {
                group_id: Set(id),
                user_id: Set(user_id),
                ..Default::default()
            }
        })).exec(&txn).await?;
        
        txn.commit().await?;

        Ok(())
    }

    pub async fn revoke_user(&self, id: i32, user_id: i32) -> Result<(), AppError> {
        if user_groups::Entity::find_by_id(id)
            .one(&self.app_state.db)
            .await?
            .is_none()
            {
                return Err(not_found!("group not found".to_string()))
            }
        
        user_group_members::Entity::delete_many()
            .filter(user_group_members::Column::GroupId.eq(id)
                .and(user_group_members::Column::UserId.eq(user_id)))
            .exec(&self.app_state.db).await?;
        
        Ok(())
    }

    // pub async fn get_group_users(&self, id: i32) -> Result<Vec<UserResponse>, AppError> {
    //     let group = user_groups::Entity::find_by_id(id)
    //         .one(&self.app_state.db)
    //         .await?
    //         .ok_or(AppError::NotFound("group not found".to_string()))?;
    //
    //     let users = group.find_related(UserEntity)
    //         .all(&self.app_state.db)
    //         .await?
    //         .into_iter()
    //         .map(|user| UserResponse::from(user))
    //         .collect();
    //
    //     Ok(users)
    // }

    pub async fn get_group_roles(&self, id: i32) -> Result<Vec<GroupRoleResponse>, AppError> {
        if user_groups::Entity::find_by_id(id)
            .one(&self.app_state.db)
            .await?
            .is_none(){
            return Err(not_found!("group not found".to_string()))
        };

        let group_roles = roles::Entity::find()
            .select_only()
            .column_as(roles::Column::RoleId, "id")
            .column_as(roles::Column::RoleName, "name")
            .join(JoinType::InnerJoin, roles::Relation::GroupRoles.def())
            .filter(group_roles::Column::GroupId.eq(id))
            .into_model::<GroupRoleResponse>()
            .all(&self.app_state.db)
            .await?;

        Ok(group_roles)
    }

    pub async fn assign_roles(&self,
                              current_user: CurrentUser,
                              context: CedarContext,
                              group_id: i32,
                              dto: AssignRolesDto) -> Result<(), AppError> {

        self.role_service.validate_role_id(&vec![dto.role_id]).await?;

        let roles_es = self.role_service.get_role_entities(&vec![dto.role_id]).await?;
        let groups_es = self.get_group_entities(&vec![group_id]).await?;

        self.app_state
            .auth_service
            .check_permission_with_entities(
                current_user.clone(),
                context.clone(),
                AuthAction::AssignRole,
                ResourceType::Role(Some(dto.role_id)),
                roles_es.clone()
            ).await?;

        self.app_state
            .auth_service
            .check_permission_with_entities(
                current_user,
                context,
                AuthAction::AssignRole,
                ResourceType::Group(Some(group_id)),
                groups_es
            ).await?;

        let txn = self.app_state.db.begin().await?;
        if user_groups::Entity::find_by_id(group_id)
            .one(&txn)
            .await?
            .is_none()
        {
            return Err(not_found!("group not found".to_string()))
        };
        
        group_roles::Entity::delete_many().filter(group_roles::Column::GroupId.eq(group_id)).exec(&txn).await?;

        group_roles::Entity::insert(
            group_roles::ActiveModel {
                group_id: Set(group_id),
                role_id: Set(dto.role_id),
                ..Default::default()
        })
        .exec(&txn).await?;
        
        txn.commit().await?;

        Ok(())
    }


    pub async fn revoke_roles(&self,
                              current_user: CurrentUser,
                              context: CedarContext,
                              group_id: i32,
                              role_id: i32
    ) -> Result<(), AppError> {
        self.role_service.validate_role_id(&vec![role_id]).await?;
        let roles_es = self.role_service.get_role_entities(&vec![role_id]).await?;
        let groups_es = self.get_group_entities(&vec![group_id]).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                current_user.clone(),
                context.clone(),
                AuthAction::RevokeRole,
                ResourceType::Role(Some(role_id)),
                roles_es.clone()
            ).await?;

        self.app_state
            .auth_service
            .check_permission_with_entities(
                current_user,
                context,
                AuthAction::RevokeRole,
                ResourceType::Group(Some(group_id)),
                groups_es
            ).await?;

        group_roles::Entity::delete_many()
            .filter(
                Condition::all()
                    .add(
                        group_roles::Column::GroupId.eq(group_id)
                    )
                    .add(
                        group_roles::Column::RoleId.eq(role_id)
                    )
            ).exec(&self.app_state.db).await?;

        Ok(())
    }

}