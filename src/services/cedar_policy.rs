use crate::config::state::AppState;
use crate::entity::{cedar_policy_set, users};
use crate::errors::app_error::AppError;
use crate::schemas::auth::CurrentUser;
use crate::schemas::cedar_policy::{CedarContext,
                                   CedarPolicyResponse,
                                   CreatePolicyDto,
                                   QueryParams};
use crate::utils::cedar_utils::{AuthAction, ResourceType, ENTITY_TYPE_POLICY, ENTITY_ATTR_NAME};
use crate::{bad_request, not_found};
use cedar_policy::{Entities, Entity, EntityId, EntityTypeName, EntityUid, Policy, RestrictedExpression};
use core::str::FromStr;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, JoinType, PaginatorTrait, QueryFilter, QuerySelect, RelationTrait, Select, Set};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use crate::utils::function::reload_policies_and_schema;


#[derive(Clone)]
pub struct CedarPolicyService {
    app_state: AppState,
}

impl CedarPolicyService {
    pub fn new(state: AppState) -> Self {
        Self { app_state: state }
    }

    async fn get_policy_entities(&self, policy_id: i32)  -> Result<Entities, AppError> {
        let policy = cedar_policy_set::Entity::find()
            .filter(cedar_policy_set::Column::PolicyId.eq(policy_id))
            .one(&self.app_state.db)
            .await?;
        let entities: Entities = match policy {
            Some(policy) => {
                let mut entities = HashSet::new();

                let policy_uid = EntityId::from_str(&*policy.policy_id.to_string())?;
                let policy_typename = EntityTypeName::from_str(ENTITY_TYPE_POLICY)?;
                let policy_e_uid = EntityUid::from_type_name_and_id(policy_typename, policy_uid);

                let mut attrs = HashMap::new();
                let name_exp = RestrictedExpression::new_string(policy.policy_str_id);
                attrs.insert(ENTITY_ATTR_NAME.to_string(), name_exp);

                let parents = HashSet::new();
                let policy_entity = Entity::new(policy_e_uid, attrs, parents)?;
                entities.insert(policy_entity);

                let schema = self.app_state
                    .auth_service
                    .get_schema_copy().await;

                Entities::from_entities(entities, Some(&schema))?
            },
            None => {
                Err(not_found!("Cedar policy not found"))?
            }
        };

        Ok(entities)
    }

    pub async fn list_policies(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        params: QueryParams,
    ) -> Result<(Vec<Value>, u64), AppError> {
        self.app_state
            .auth_service
            .check_permission(
                current_user.user_id,
                context,
                AuthAction::ViewPolicy,
                ResourceType::Policy(None),
            )
            .await?;

        let mut query = cedar_policy_set::Entity::find()
            .column_as(users::Column::Email, "created_user")
            .join(
                JoinType::InnerJoin,
                cedar_policy_set::Relation::Users.def()
            );
        if let Some(effect) = &params.effect {
            query = query.filter(cedar_policy_set::Column::Effect.eq(effect));
        }
        if let Some(is_active) = &params.is_active {
            query = query.filter(cedar_policy_set::Column::IsActive.eq(*is_active));
        }

        let select: Select<cedar_policy_set::Entity> = match params.fields.as_ref() {
            Some(fields) => self.list_policy_with_fields(query, fields).await?,
            None => query,
        };

        let paginator = select
            .into_model::<CedarPolicyResponse>()
            .paginate(&self.app_state.db, params.page_size);
        let total = paginator.num_items().await?;
        let results = paginator
            .fetch_page(params.page - 1)
            .await?
            .into_iter()
            .map(|x| serde_json::to_value(x))
            .collect::<Result<Vec<_>, _>>()?;

        Ok((results, total))
    }

    async fn list_policy_with_fields(
        &self,
        query: Select<cedar_policy_set::Entity>,
        fields: &str,
    ) -> Result<Select<cedar_policy_set::Entity>, AppError> {
        let requested_fields: Vec<&str> = fields.split(',').map(|s| s.trim()).collect();
        let mut select = query.select_only();
        for field in &requested_fields {
            select = match *field {
                "id" => select.column_as(cedar_policy_set::Column::PolicyId, "id"),
                "policy_text" => select.column(cedar_policy_set::Column::PolicyText),
                "effect" => select.column(cedar_policy_set::Column::Effect),
                "is_active" => select.column(cedar_policy_set::Column::IsActive),
                "description" => select.column(cedar_policy_set::Column::Description),
                _ => select,
            }
        }
        Ok(select)
    }

    pub async fn get_policy(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        policy_id: i32,
    ) -> Result<CedarPolicyResponse, AppError> {
        let es = self.get_policy_entities(policy_id).await?;

        self.app_state
            .auth_service
            .check_permission_with_entities(
                current_user.user_id,
                context,
                AuthAction::ViewPolicy,
                ResourceType::Policy(Some(policy_id)),
                es
            ).await?;

        let policy_model = cedar_policy_set::Entity::find_by_id(policy_id)
            .column_as(users::Column::Email, "email")
            .join(
                JoinType::InnerJoin,
                cedar_policy_set::Relation::Users.def()
            )
            .into_model::<CedarPolicyResponse>()
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!("Policy {} not found", policy_id))?;

        Ok(policy_model)
    }

    pub async fn create_policy(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        dto: CreatePolicyDto,
    ) -> Result<CedarPolicyResponse, AppError> {

        self.app_state
            .auth_service
            .check_permission(
                current_user.user_id,
                context,
                AuthAction::CreatePolicy,
                ResourceType::Policy(None),
            ).await?;

        let policy = Policy::from_str(dto.policy_text.as_str())?;
        let policy_str_id = policy.annotation("id").ok_or(
            bad_request!("Missing id annotation for policy")
        )?.to_string();
        let effect = policy.effect().to_string();
        let policy_hash = Self::hash_policy_content(&policy.to_string())?;

        let new_policy_model = cedar_policy_set::ActiveModel {
            policy_str_id: Set(policy_str_id),
            policy_text: Set(dto.policy_text),
            policy_hash: Set(policy_hash),
            effect: Set(effect),
            is_active: Set(dto.is_active),
            description: Set(dto.description),
            created_by: Set(current_user.user_id),
            ..Default::default()
        };

        let new_model = new_policy_model.insert(&self.app_state.db)
            .await?;

        let creator = users::Entity::find_by_id(new_model.created_by)
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!("User {} not found", current_user.user_id))?;

        let response = CedarPolicyResponse{
            policy_id: Some(new_model.policy_id),
            policy_str_id: Some(new_model.policy_str_id),
            policy_text: Some(new_model.policy_text),
            effect: Some(new_model.effect),
            is_active: Some(new_model.is_active),
            description: Some(new_model.description),
            created_user: Some(creator.username),
            created_at: new_model.created_at,
            updated_at: new_model.updated_at,
        };

        Ok(response)
    }

    fn hash_policy_content(policy_text: &String) -> Result<String, AppError> {
        let mut hasher = Sha256::new();
        hasher.update(policy_text.as_bytes());
        let hash_bytes = hasher.finalize();
        Ok(format!("{:x}", hash_bytes))
    }

    pub async fn update_policy(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        policy_id: i32,
        dto: CreatePolicyDto,
    ) -> Result<CedarPolicyResponse, AppError> {
        let user_id = current_user.user_id;
        let es = self.get_policy_entities(policy_id).await?;
        self.app_state
        .auth_service
            .check_permission_with_entities(
                current_user.user_id,
                context,
                AuthAction::UpdatePolicy,
                ResourceType::Policy(Some(policy_id)),
                es
            ).await?;

        let policy = Policy::from_str(dto.policy_text.as_str())?;
        let policy_str_id = policy.id().to_string();
        let effect = policy.effect().to_string();
        let policy_hash = Self::hash_policy_content(&policy.to_string())?;

        let mut policy_model: cedar_policy_set::ActiveModel = cedar_policy_set::Entity::find_by_id(policy_id)
        .one(&self.app_state.db)
        .await?
        .ok_or(not_found!("Policy {} not found", policy_id))?
            .into();

        policy_model.policy_str_id = Set(policy_str_id);
        policy_model.policy_text = Set(dto.policy_text);
        policy_model.effect = Set(effect);
        policy_model.is_active = Set(dto.is_active);
        policy_model.description = Set(dto.description);
        policy_model.created_by= Set(user_id);
        policy_model.policy_hash = Set(policy_hash);

        let new_model = policy_model.update(&self.app_state.db).await?;

        let creator = users::Entity::find_by_id(new_model.created_by)
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!("User {} not found", user_id))?;

        let response = CedarPolicyResponse{
            policy_id: Some(new_model.policy_id),
            policy_str_id: Some(new_model.policy_str_id),
            policy_text: Some(new_model.policy_text),
            effect: Some(new_model.effect),
            is_active: Some(new_model.is_active),
            description: Some(new_model.description),
            created_user: Some(creator.username),
            created_at: new_model.created_at,
            updated_at: new_model.updated_at,
        };

        Ok(response)
    }

    pub async fn delete_policy(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        policy_id: i32,
    ) -> Result<(), AppError> {
        let es = self.get_policy_entities(policy_id).await?;
        self.app_state
        .auth_service
            .check_permission_with_entities(
                current_user.user_id,
                context,
                AuthAction::DeletePolicy,
                ResourceType::Policy(Some(policy_id)),
                es
            ).await?;

        cedar_policy_set::Entity::delete_by_id(policy_id).exec(&self.app_state.db).await?;

        Ok(())
    }
    
    pub async fn update_policies_cache(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
    ) -> Result<(), AppError> {
        self.app_state
            .auth_service
            .check_permission(
                current_user.user_id,
                context,
                AuthAction::UpdatePolicy,
                ResourceType::Policy(None),
            ).await?;

        reload_policies_and_schema(&self.app_state).await?;
        
        Ok(())
    }
}
