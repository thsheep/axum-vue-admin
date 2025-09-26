use crate::config::state::AppState;
use crate::entity::{cedar_policy_set, users};
use crate::errors::app_error::AppError;
use crate::schemas::auth::CurrentUser;
use crate::schemas::cedar_policy::{CedarContext,
                                   CedarPolicyResponse,
                                   CreatePolicyDto,
                                   QueryParams};
use crate::utils::cedar_utils::{AuthAction, ResourceType, ENTITY_TYPE_POLICY, ENTITY_ATTR_NAME};
use crate::{bad_request, conflict, not_found};
use cedar_policy::{Entities, Entity, EntityId, EntityTypeName, EntityUid, Policy, RestrictedExpression};
use core::str::FromStr;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, JoinType, PaginatorTrait, QueryFilter, QuerySelect, RelationTrait, Select, Set};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use crate::utils::function::reload_policies_and_schema;


#[derive(Clone)]
pub struct CedarPolicyService {
    app_state: AppState,
}

impl CedarPolicyService {
    pub fn new(state: AppState) -> Self {
        Self { app_state: state }
    }

    async fn get_policy_entities(&self, policy_uuid: &str) -> Result<Entities, AppError> {
        let policy = cedar_policy_set::Entity::find()
            .filter(cedar_policy_set::Column::PolicyUuid.eq(policy_uuid))
            .one(&self.app_state.db)
            .await?;
        let entities: Entities = match policy {
            Some(policy) => {
                let mut entities = HashSet::new();

                let policy_uid = EntityId::from_str(&*policy.policy_uuid.to_string())?;
                let policy_typename = EntityTypeName::from_str(ENTITY_TYPE_POLICY)?;
                let policy_e_uid = EntityUid::from_type_name_and_id(policy_typename, policy_uid);

                let mut attrs = HashMap::new();
                let name_exp = RestrictedExpression::new_string(policy.annotation);
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
                &current_user.uuid,
                context,
                AuthAction::ViewPolicy,
                ResourceType::Policy(None),
            )
            .await?;

        let requested_fields: HashSet<String> = params
            .fields
            .map(|f| f.split(',').map(|s| s.trim().to_string()).collect()).unwrap_or_else(|| {
            ["uuid", "annotation",  "policy_type", "effect", "is_active", "description",
                "created_user", "created_at", "updated_at"]
                .iter().map(|s| s.to_string()).collect()
        });

        let mut query = cedar_policy_set::Entity::find()
            .join(JoinType::InnerJoin, cedar_policy_set::Relation::Users.def());

        if let Some(effect) = &params.effect {
            query = query.filter(cedar_policy_set::Column::Effect.eq(effect));
        }
        if let Some(is_active) = &params.is_active {
            query = query.filter(cedar_policy_set::Column::IsActive.eq(*is_active));
        }

        let mut select = query.select_only();
        for field in &requested_fields {
            select = match field.as_str() {
                "uuid" => select.column_as(cedar_policy_set::Column::PolicyUuid, "uuid"),
                "annotation" => select.column(cedar_policy_set::Column::Annotation),
                "created_user" => select.column_as(users::Column::Email, "created_user"),
                "policy_text" => select.column(cedar_policy_set::Column::PolicyText),
                "policy_type" => select.column(cedar_policy_set::Column::PolicyType),
                "effect" => select.column(cedar_policy_set::Column::Effect),
                "is_active" => select.column(cedar_policy_set::Column::IsActive),
                "description" => select.column(cedar_policy_set::Column::Description),
                _ => select,
            };
        }

        let paginator = select.into_json()
            .paginate(&self.app_state.db, params.page_size);
        let total = paginator.num_items().await?;
        let page_index = if params.page > 0 { params.page - 1 } else { 0 };
        let results = paginator.fetch_page(page_index).await?;

        Ok((results, total))
    }

    pub async fn get_policy(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        policy_uuid: String,
    ) -> Result<CedarPolicyResponse, AppError> {
        let es = self.get_policy_entities(&policy_uuid).await?;

        self.app_state
            .auth_service
            .check_permission_with_entities(
                &current_user.uuid,
                context,
                AuthAction::ViewPolicy,
                ResourceType::Policy(Some(policy_uuid.clone())),
                es
            ).await?;

        let result = cedar_policy_set::Entity::find()
            .filter(cedar_policy_set::Column::PolicyUuid.eq(&policy_uuid))
            .find_also_related(users::Entity)
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!(format!("Policy with UUID '{}' not found", policy_uuid)))?;

        let (policy, user_opt) = result;
        let created_user = user_opt.map(|u| u.username).unwrap_or_else(|| "Unknown".to_string());

        Ok(CedarPolicyResponse{
            uuid: Some(policy.policy_uuid),
            annotation: Some(policy.annotation),
            policy_text: Some(policy.policy_text),
            policy_type: Some(policy.policy_type),
            effect: Some(policy.effect),
            is_active: Some(policy.is_active),
            description: Some(policy.description),
            created_user: Some(created_user),
            created_at: policy.created_at,
            updated_at: policy.updated_at
        })
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
                &current_user.uuid,
                context,
                AuthAction::CreatePolicy,
                ResourceType::Policy(None),
            ).await?;

        let policy = Policy::from_str(&dto.policy_text)?;
        let annotation = policy.annotation("annotation").ok_or(bad_request!("Policy must have an 'annotation'"))?.to_string();
        let effect = policy.effect().to_string();
        let policy_hash = Self::hash_policy_content(&policy.to_string())?;
        if cedar_policy_set::Entity::find()
            .filter(cedar_policy_set::Column::PolicyHash.eq(&policy_hash))
            .one(&self.app_state.db)
            .await?
            .is_some()
            {
                return Err(conflict!("A policy with the exact same content already exists."));
            }

        let (email, user_id) = users::Entity::find()
            .select_only()
            .column(users::Column::Email)
            .column(users::Column::UserId)
            .filter(users::Column::UserUuid.eq(&current_user.uuid))
            .into_tuple::<(String, i32)>()
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!("User {} not found", current_user.uuid))?;
        
        let policy_uuid = Uuid::new_v4().to_string();
        let new_policy_model = cedar_policy_set::ActiveModel {
            annotation: Set(annotation),
            policy_text: Set(dto.policy_text),
            policy_hash: Set(policy_hash),
            effect: Set(effect),
            policy_type: Set(dto.policy_type),
            policy_uuid:Set(policy_uuid),
            is_active: Set(dto.is_active),
            description: Set(dto.description),
            created_by: Set(user_id),
            ..Default::default()
        };

        let new_model = new_policy_model.insert(&self.app_state.db)
            .await?;
        
        let response = CedarPolicyResponse{
            uuid: Some(new_model.policy_uuid),
            annotation: Some(new_model.annotation),
            policy_text: Some(new_model.policy_text),
            policy_type: Some(new_model.policy_type),
            effect: Some(new_model.effect),
            is_active: Some(new_model.is_active),
            description: Some(new_model.description),
            created_user: Some(email),
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
        policy_uuid: String,
        dto: CreatePolicyDto,
    ) -> Result<CedarPolicyResponse, AppError> {
        let es = self.get_policy_entities(&policy_uuid).await?;
        self.app_state
        .auth_service
            .check_permission_with_entities(
                &current_user.uuid,
                context,
                AuthAction::UpdatePolicy,
                ResourceType::Policy(Some(policy_uuid.clone())),
                es
            ).await?;

        let policy = Policy::from_str(dto.policy_text.as_str())?;
        let annotation = policy.annotation("annotation").ok_or(
            bad_request!("Missing annotation for policy")
        )?.to_string();
        let effect = policy.effect().to_string();
        let policy_hash = Self::hash_policy_content(&policy.to_string())?;

        let mut policy_model: cedar_policy_set::ActiveModel = cedar_policy_set::Entity::find()
            .filter(cedar_policy_set::Column::PolicyUuid.eq(&policy_uuid))
            .one(&self.app_state.db)
        .await?
        .ok_or(not_found!("Policy {} not found", policy_uuid))?
            .into();
        
        let user_id = users::Entity::find()
            .select_only()
            .column(users::Column::UserId)
            .filter(users::Column::UserUuid.eq(&current_user.uuid))
            .into_tuple::<i32>()
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!("User {} not found", current_user.uuid))?;
        
        policy_model.annotation = Set(annotation);
        policy_model.policy_text = Set(dto.policy_text);
        policy_model.effect = Set(effect);
        policy_model.policy_type = Set(dto.policy_type);
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
            uuid: Some(new_model.policy_uuid),
            annotation: Some(new_model.annotation),
            policy_text: Some(new_model.policy_text),
            policy_type: Some(new_model.policy_type),
            effect: Some(new_model.effect),
            is_active: Some(new_model.is_active),
            description: Some(new_model.description),
            created_user: Some(creator.email),
            created_at: new_model.created_at,
            updated_at: new_model.updated_at,
        };

        Ok(response)
    }

    pub async fn delete_policy(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        policy_uuid: String,
    ) -> Result<(), AppError> {
        let es = self.get_policy_entities(&policy_uuid).await?;
        self.app_state
        .auth_service
            .check_permission_with_entities(
                &current_user.uuid,
                context,
                AuthAction::DeletePolicy,
                ResourceType::Policy(Some(policy_uuid.clone())),
                es
            ).await?;

        cedar_policy_set::Entity::delete_many()
            .filter(cedar_policy_set::Column::PolicyUuid.eq(policy_uuid))
            .exec(&self.app_state.db).await?;

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
                &current_user.uuid,
                context,
                AuthAction::UpdatePolicy,
                ResourceType::Policy(None),
            ).await?;

        reload_policies_and_schema(&self.app_state).await?;
        
        Ok(())
    }
}
