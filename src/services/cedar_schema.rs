use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use crate::{bad_request, not_found};
use crate::config::state::AppState;
use crate::entity::cedar_schema;
use crate::errors::app_error::AppError;
use crate::schemas::auth::CurrentUser;
use crate::schemas::cedar_policy::{CedarContext, CedarSchemaResponse, UpdateSchema};
use crate::utils::cedar_utils::{AuthAction, ResourceType};

#[derive(Clone)]
pub struct CedarSchemaService {
    app_state: AppState,
}

impl CedarSchemaService {
    pub fn new(state: AppState) -> Self {
        Self { app_state: state }
    }

    pub async fn list_schema(
        &self,
        current_user: CurrentUser,
        context: CedarContext
    ) -> Result<Vec<CedarSchemaResponse>, AppError> {

        self.app_state
            .auth_service
            .check_permission(
                &current_user.uuid,
                context,
                AuthAction::ViewPolicy,
                ResourceType::Policy(None),
            ).await?;

        let model = cedar_schema::Entity::find()
            .one(&self.app_state.db)
            .await?
            .ok_or(bad_request!("Not found Schema"))?;

        let response = CedarSchemaResponse{
            uuid: model.schema_uuid,
            schema: model.schema,
            description: model.description,
            created_at: model.created_at,
            updated_at: model.updated_at,
        };

        Ok(vec![response])
    }

    pub async fn update_schema(
        &self,
        current_user: CurrentUser,
        context: CedarContext,
        schema_id: i32,
        dto: UpdateSchema
    ) -> Result<CedarSchemaResponse, AppError> {
        self.app_state
        .auth_service
        .check_permission(
            &current_user.uuid,
            context,
            AuthAction::UpdatePolicy,
            ResourceType::Policy(None),
        ).await?;

        let mut schema: cedar_schema::ActiveModel = cedar_schema::Entity::find_by_id(schema_id)
            .one(&self.app_state.db)
            .await?
            .ok_or(not_found!("Schema {} not found", schema_id))?
            .into();

        schema.schema=Set(dto.schema);
        schema.description=Set(dto.description);

        let new_model = schema.update(&self.app_state.db).await?;

        let response = CedarSchemaResponse{
            uuid: new_model.schema_uuid,
            schema: new_model.schema,
            description: new_model.description,
            created_at: new_model.created_at,
            updated_at: new_model.updated_at,
        };
        Ok(response)
    }
}