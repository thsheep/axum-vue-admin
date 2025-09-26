use crate::config::state::AppState;
use crate::entity::{group_roles, roles, user_group_members, user_groups, users};

use crate::errors::app_error::AppError;
use crate::schemas::auth::CurrentUser;
use crate::schemas::cedar_policy::CedarContext;
use crate::schemas::groups::AssignRolesDto;
use crate::schemas::groups::{
    AssignUsersDto, CreateGroupDto, GroupResponse, GroupRoleResponse, QueryParams,
};
use crate::services::role::get_role_entities;
use crate::utils::cedar_utils::{
    AuthAction, ENTITY_ATTR_NAME, ENTITY_TYPE_GROUP, ResourceType, entities2json,
};
use crate::{bad_request, conflict, not_found};
use cedar_policy::{
    Entities, Entity, EntityId, EntityTypeName, EntityUid, RestrictedExpression, Schema,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, JoinType,
    ModelTrait, PaginatorTrait, QueryFilter, QuerySelect, RelationTrait, Select, Set,
    TransactionTrait,
};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use tracing::debug;
use uuid::Uuid;

#[derive(Clone)]
pub struct GroupService {
    app_state: AppState,
}

impl GroupService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
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
                &current_user.uuid,
                context,
                AuthAction::ViewGroup,
                ResourceType::Group(None),
            )
            .await?;

        let requested_fields: HashSet<String> = params
            .fields
            .map(|f| f.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|| {
                // 默认返回所有字段
                ["id", "name", "description", "created_at", "updated_at"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            });

        let mut query = user_groups::Entity::find();
        if let Some(name) = &params.name {
            query = query.filter(user_groups::Column::Name.contains(name));
        }

        let mut select = query.select_only();
        for field in &requested_fields {
            select = match field.as_str() {
                "id" => select.column_as(user_groups::Column::UserGroupUuid, "uuid"),
                "name" => select.column(user_groups::Column::Name),
                "description" => select.column(user_groups::Column::Description),
                "created_at" => select.column(user_groups::Column::CreatedAt),
                "updated_at" => select.column(user_groups::Column::UpdatedAt),
                _ => select,
            };
        }

        let paginator = select
            .into_json()
            .paginate(&self.app_state.db, params.page_size);
        let total = paginator.num_items().await?;
        let page_index = if params.page > 0 { params.page - 1 } else { 0 };
        let results = paginator.fetch_page(page_index).await?;

        Ok((results, total))
    }

    pub async fn create_group(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        create_group_dto: CreateGroupDto,
    ) -> Result<GroupResponse, AppError> {
        self.app_state
            .auth_service
            .check_permission(
                &current_user.uuid,
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
            user_group_uuid: Set(Uuid::new_v4().to_string()),
            name: Set(create_group_dto.name),
            description: Set(create_group_dto.description),
            ..Default::default()
        };

        let user_group = insert_model.insert(&txn).await?;

        txn.commit().await?;
        Ok(GroupResponse::from(user_group))
    }

    pub async fn get_group(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        group_uuid: String,
    ) -> Result<GroupResponse, AppError> {
        let schema = self.app_state.auth_service.get_schema_copy().await;
        let es = get_group_entities(&self.app_state.db, &vec![group_uuid.clone()], &schema).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                &current_user.uuid,
                context,
                AuthAction::ViewGroup,
                ResourceType::Group(Some(group_uuid.clone())),
                es,
            )
            .await?;

        let group = user_groups::Entity::find()
            .filter(user_groups::Column::UserGroupUuid.eq(group_uuid))
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!("group not found".to_string()))?;

        Ok(GroupResponse::from(group))
    }

    pub async fn update_group(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        group_uuid: String,
        update_group_dto: CreateGroupDto,
    ) -> Result<GroupResponse, AppError> {
        let schema = self.app_state.auth_service.get_schema_copy().await;
        let es = get_group_entities(&self.app_state.db, &vec![group_uuid.clone()], &schema).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                &current_user.uuid,
                context,
                AuthAction::UpdateGroup,
                ResourceType::Group(Some(group_uuid.clone())),
                es,
            )
            .await?;

        let mut group: user_groups::ActiveModel = user_groups::Entity::find()
            .filter(user_groups::Column::UserGroupUuid.eq(group_uuid))
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!("group not found".to_string()))?
            .into();

        group.name = Set(update_group_dto.name);
        group.description = Set(update_group_dto.description);

        let group = group.update(&self.app_state.db).await?;

        Ok(GroupResponse::from(group))
    }

    pub async fn delete_group(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        group_uuid: String,
    ) -> Result<(), AppError> {
        let schema = self.app_state.auth_service.get_schema_copy().await;
        let es = get_group_entities(&self.app_state.db, &vec![group_uuid.clone()], &schema).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                &current_user.uuid,
                context,
                AuthAction::DeleteGroup,
                ResourceType::Group(Some(group_uuid.clone())),
                es,
            )
            .await?;

        let txn = self.app_state.db.begin().await?;

        let group_id = user_groups::Entity::find()
            .select_only()
            .column(user_groups::Column::UserGroupId)
            .into_tuple::<i32>()
            .one(&txn)
            .await?
            .ok_or(not_found!("group not found".to_string()))?;

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

    pub async fn assign_users(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        group_uuid: String,
        dto: AssignUsersDto,
    ) -> Result<(), AppError> {
        let schema = self.app_state.auth_service.get_schema_copy().await;
        let groups_es =
            get_group_entities(&self.app_state.db, &vec![group_uuid.clone()], &schema).await?;

        self.app_state
            .auth_service
            .check_permission_with_entities(
                &current_user.uuid,
                context,
                AuthAction::CreateUser,
                ResourceType::Group(Some(group_uuid.clone())),
                groups_es,
            )
            .await?;

        let txn = self.app_state.db.begin().await?;
        let group_id = user_groups::Entity::find()
            .select_only()
            .column(user_groups::Column::UserGroupId)
            .filter(user_groups::Column::UserGroupUuid.eq(group_uuid))
            .into_tuple::<i32>()
            .one(&txn)
            .await?
            .ok_or(not_found!("group not found".to_string()))?;

        user_group_members::Entity::delete_many()
            .filter(user_group_members::Column::GroupId.eq(group_id))
            .exec(&txn)
            .await?;

        if dto.user_uuids.is_empty() {
            txn.commit().await?;
            return Ok(());
        }

        let user_ids = users::Entity::find()
            .select_only()
            .column(users::Column::UserId)
            .filter(users::Column::UserUuid.is_in(&dto.user_uuids))
            .into_tuple::<i32>()
            .all(&txn)
            .await?;

        if user_ids.len() != dto.user_uuids.len() {
            return Err(bad_request!("User mismatch"));
        }

        user_group_members::Entity::insert_many(user_ids.into_iter().map(|user_id| {
            user_group_members::ActiveModel {
                group_id: Set(group_id),
                user_id: Set(user_id),
                ..Default::default()
            }
        }))
        .exec(&txn)
        .await?;

        txn.commit().await?;

        Ok(())
    }

    pub async fn revoke_user(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        group_uuid: String,
        user_uuid: String,
    ) -> Result<(), AppError> {
        let schema = self.app_state.auth_service.get_schema_copy().await;
        let groups_es =
            get_group_entities(&self.app_state.db, &vec![group_uuid.clone()], &schema).await?;

        self.app_state
            .auth_service
            .check_permission_with_entities(
                &current_user.uuid,
                context,
                AuthAction::DeleteUser,
                ResourceType::Group(Some(group_uuid.clone())),
                groups_es,
            )
            .await?;

        let group_id = user_groups::Entity::find()
            .select_only()
            .column(user_groups::Column::UserGroupId)
            .filter(user_groups::Column::UserGroupUuid.eq(group_uuid))
            .into_tuple::<i32>()
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!("group not found".to_string()))?;

        let user_id_to_revoke = users::Entity::find()
            .select_only()
            .column(users::Column::UserId)
            .filter(users::Column::UserUuid.eq(&user_uuid))
            .into_tuple::<i32>()
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!("User to revoke not found"))?;

        user_group_members::Entity::delete_many()
            .filter(
                user_group_members::Column::GroupId
                    .eq(group_id)
                    .and(user_group_members::Column::UserId.eq(user_id_to_revoke)),
            )
            .exec(&self.app_state.db)
            .await?;

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

    pub async fn get_group_roles(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        group_uuid: String,
    ) -> Result<Vec<GroupRoleResponse>, AppError> {
        let schema = self.app_state.auth_service.get_schema_copy().await;
        let groups_es =
            get_group_entities(&self.app_state.db, &vec![group_uuid.clone()], &schema).await?;

        self.app_state
            .auth_service
            .check_permission_with_entities(
                &current_user.uuid,
                context,
                AuthAction::ViewRole,
                ResourceType::Group(Some(group_uuid.clone())),
                groups_es,
            )
            .await?;

        let group_id = user_groups::Entity::find()
            .select_only()
            .column(user_groups::Column::UserGroupId)
            .filter(user_groups::Column::UserGroupUuid.eq(&group_uuid))
            .into_tuple::<i32>()
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!("group not found".to_string()))?;

        let group_roles = roles::Entity::find()
            .select_only()
            .column_as(roles::Column::RoleUuid, "uuid")
            .column_as(roles::Column::RoleName, "name")
            .join(JoinType::InnerJoin, roles::Relation::GroupRoles.def())
            .filter(group_roles::Column::GroupId.eq(group_id))
            .into_model::<GroupRoleResponse>()
            .all(&self.app_state.db)
            .await?;

        Ok(group_roles)
    }

    pub async fn assign_roles(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        group_uuid: String,
        dto: AssignRolesDto,
    ) -> Result<(), AppError> {
        let schema = self.app_state.auth_service.get_schema_copy().await;
        let role_es =
            get_role_entities(&self.app_state.db, &vec![dto.role_uuid.clone()], &schema).await?;
        let groups_es =
            get_group_entities(&self.app_state.db, &vec![group_uuid.clone()], &schema).await?;

        self.app_state
            .auth_service
            .check_permission_with_entities(
                &current_user.uuid,
                context.clone(),
                AuthAction::AssignRole,
                ResourceType::Role(Some(dto.role_uuid.clone())),
                role_es.clone(),
            )
            .await?;

        self.app_state
            .auth_service
            .check_permission_with_entities(
                &current_user.uuid,
                context,
                AuthAction::AssignRole,
                ResourceType::Group(Some(group_uuid.clone())),
                groups_es,
            )
            .await?;

        let txn = self.app_state.db.begin().await?;
        let group_id = user_groups::Entity::find()
            .select_only()
            .column(user_groups::Column::UserGroupId)
            .filter(user_groups::Column::UserGroupUuid.eq(&group_uuid))
            .into_tuple::<i32>()
            .one(&txn)
            .await?
            .ok_or(not_found!("group not found".to_string()))?;

        let role_id = roles::Entity::find()
            .select_only()
            .column(roles::Column::RoleId)
            .filter(roles::Column::RoleUuid.eq(&dto.role_uuid))
            .into_tuple::<i32>()
            .one(&txn)
            .await?
            .ok_or(not_found!("role not found".to_string()))?;

        group_roles::Entity::delete_many()
            .filter(group_roles::Column::GroupId.eq(group_id))
            .exec(&txn)
            .await?;

        group_roles::Entity::insert(group_roles::ActiveModel {
            group_id: Set(group_id),
            role_id: Set(role_id),
            ..Default::default()
        })
        .exec(&txn)
        .await?;

        txn.commit().await?;

        Ok(())
    }

    pub async fn revoke_roles(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        group_uuid: String,
        role_uuid: String,
    ) -> Result<(), AppError> {
        let schema = self.app_state.auth_service.get_schema_copy().await;
        let role_es =
            get_role_entities(&self.app_state.db, &vec![role_uuid.clone()], &schema).await?;
        let groups_es =
            get_group_entities(&self.app_state.db, &vec![group_uuid.clone()], &schema).await?;
        self.app_state
            .auth_service
            .check_permission_with_entities(
                &current_user.uuid,
                context.clone(),
                AuthAction::RevokeRole,
                ResourceType::Role(Some(role_uuid.clone())),
                role_es.clone(),
            )
            .await?;

        self.app_state
            .auth_service
            .check_permission_with_entities(
                &current_user.uuid,
                context,
                AuthAction::RevokeRole,
                ResourceType::Group(Some(group_uuid.clone())),
                groups_es,
            )
            .await?;

        let group_id = user_groups::Entity::find()
            .select_only()
            .column(user_groups::Column::UserGroupId)
            .filter(user_groups::Column::UserGroupUuid.eq(&group_uuid))
            .into_tuple::<i32>()
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!("group not found".to_string()))?;

        let role_id = roles::Entity::find()
            .select_only()
            .column(roles::Column::RoleId)
            .filter(roles::Column::RoleUuid.eq(&role_uuid))
            .into_tuple::<i32>()
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!("role not found".to_string()))?;

        group_roles::Entity::delete_many()
            .filter(
                Condition::all()
                    .add(group_roles::Column::GroupId.eq(group_id))
                    .add(group_roles::Column::RoleId.eq(role_id)),
            )
            .exec(&self.app_state.db)
            .await?;

        Ok(())
    }
}

// 获取用户组实体信息
pub async fn get_group_entities(
    db: &DatabaseConnection,
    group_uuids: &[String],
    schema: &Schema,
) -> Result<Entities, AppError> {
    let groups = user_groups::Entity::find()
        .column(user_groups::Column::Name)
        .filter(user_groups::Column::UserGroupUuid.is_in(group_uuids.to_vec()))
        .all(db)
        .await?;

    let mut entities = HashSet::new();
    for group in groups {
        let group_eid = EntityId::from_str(&group.user_group_uuid.to_string())?;
        let group_typename = EntityTypeName::from_str(ENTITY_TYPE_GROUP)?;
        let group_e_uid = EntityUid::from_type_name_and_id(group_typename, group_eid);

        let mut attrs = HashMap::new();
        let name_expr = RestrictedExpression::new_string(group.name);
        attrs.insert(ENTITY_ATTR_NAME.to_string(), name_expr);

        let parents = HashSet::new();
        let group_entity = Entity::new(group_e_uid, attrs, parents)?;
        entities.insert(group_entity);
    }

    let verified_entities = Entities::from_entities(entities, Some(&schema))?;
    let entities_json = entities2json(&verified_entities)?;
    debug!("Groups:{:?}; Entities Json: {}", group_uuids, entities_json);
    Ok(verified_entities)
}
