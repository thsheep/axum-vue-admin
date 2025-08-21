use crate::config::openapi::ME_TAG;
use crate::errors::app_error::AppError;
use crate::schemas::{
    auth::CurrentUser,
    me::Profile, response::ApiResponse
};
use crate::services::me::MeService;
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::IntoResponse,
};
use crate::schemas::cedar_policy::CedarContext;

#[utoipa::path(get, 
    path = "/profile",
    responses((status = 200, body = Profile),),
    tag = ME_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn profile(
    State(service): State<MeService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>
) -> Result<impl IntoResponse, AppError> {
    let profile = service.profile(current_user, context).await?;
    Ok(ApiResponse::success(profile, StatusCode::OK))
}