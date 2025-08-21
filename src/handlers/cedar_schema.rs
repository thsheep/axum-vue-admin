use crate::config::openapi::CEDAR_POLICY_TAG;
use crate::errors::app_error::AppError;
use crate::schemas::auth::CurrentUser;
use crate::schemas::cedar_policy::{CedarContext, CedarPolicyResponse, CedarSchemaResponse, UpdateSchema};
use crate::schemas::response::ApiResponse;
use crate::services::cedar_schema::CedarSchemaService;
use axum::extract::Query;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use validator::Validate;


#[utoipa::path(
    get,
    path = "",
    responses((status = 200, body = Vec<CedarSchemaResponse>),),
    tag = CEDAR_POLICY_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn list_schema(
    State(service): State<CedarSchemaService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    let schemas = service.list_schema(
        current_user,
        context,
    ).await?;
    Ok(ApiResponse::success(schemas, StatusCode::OK))
}



#[utoipa::path(
    put,
    path = "/{schema_id}",
    request_body=UpdateSchema,
    params(
        ("schema_id" = i32, Path, description = "Schema唯一ID", example = 42)
    ),
    responses(( status=200, body=CedarPolicyResponse, description = "更新成功"),
                (status=404, description="策略不存在"),),
    tag = CEDAR_POLICY_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn update_schema(
    State(service): State<CedarSchemaService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
    Path(schema_id): Path<i32>,
    Json(dto): Json<UpdateSchema>,
) -> Result<impl IntoResponse, AppError> {
    dto.validate()?;
    let schemas = service.update_schema(
        current_user,
        context,
        schema_id,
        dto
    ).await?;
    Ok(ApiResponse::success(schemas, StatusCode::OK))
}