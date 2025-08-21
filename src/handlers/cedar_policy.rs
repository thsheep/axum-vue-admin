use crate::config::openapi::CEDAR_POLICY_TAG;
use crate::errors::app_error::AppError;
use crate::schemas::auth::CurrentUser;
use crate::schemas::cedar_policy::{CedarContext, CedarPolicyResponse, CreatePolicyDto, QueryParams};
use crate::schemas::paginated::PaginatedApiResponse;
use crate::schemas::response::ApiResponse;
use crate::services::cedar_policy::CedarPolicyService;
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
    params(QueryParams),
    responses((status = 200, body = Vec<CedarPolicyResponse>),),
    tag = CEDAR_POLICY_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn list_policies(
    State(service): State<CedarPolicyService>,
    Query(params): Query<QueryParams>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    params.validate()?;
    let (policies, total) = service.list_policies(
        current_user,
        context,
        params.clone(),
    ).await?;
    Ok(PaginatedApiResponse::success(policies,
                                     total,
                                     params.page,
                                     params.page_size,
                                     StatusCode::OK
    ))
}


#[utoipa::path(
    post,
    path = "",
    request_body=CreatePolicyDto,
    responses(( status=200,  description = "创建策略成功"),
                (status=409, description = "策略已存在"),),
    tag = CEDAR_POLICY_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn create_policy(
    State(service): State<CedarPolicyService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
    Json(dto): Json<CreatePolicyDto>,
) -> Result<impl IntoResponse, AppError> {
    dto.validate()?;
    let policy = service.create_policy(
        current_user,
        context,
        dto,
    ).await?;
    Ok(ApiResponse::success(policy, StatusCode::CREATED))
}


#[utoipa::path(
    get,
    path = "/{policy_id}",
    params(
        ("policy_id" = i32, Path, description = "策略唯一ID", example = 42)
    ),
    responses(( status=200, body=CedarPolicyResponse, description = "获取成功"),
    ( status=404, description = "不存在"),),
    tag = CEDAR_POLICY_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn get_policy(
    State(service): State<CedarPolicyService>,
    Path(policy_id): Path<i32>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    let policy = service.get_policy(
        current_user,
        context,
        policy_id,
    ).await?;
    
    Ok(ApiResponse::success(policy, StatusCode::OK))
}

#[utoipa::path(
    put,
    path = "/{policy_id}",
    request_body=CreatePolicyDto,
    params(
        ("policy_id" = i32, Path, description = "Policy唯一ID", example = 42)
    ),
    responses(( status=200, body=CedarPolicyResponse, description = "更新成功"),
                (status=404, description="策略不存在"),),
    tag = CEDAR_POLICY_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn update_policy(
    State(service): State<CedarPolicyService>,
    Path(policy_id): Path<i32>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
    Json(dto): Json<CreatePolicyDto>,
) -> Result<impl IntoResponse, AppError> {  
    
    dto.validate()?;
    
    let policy = service.update_policy(
        current_user,
        context,
        policy_id,
        dto
    ).await?;
    Ok(ApiResponse::success(policy, StatusCode::OK))
}


#[utoipa::path(
    delete,
    path = "/{policy_id}",
    params(
        ("policy_id" = i32, Path, description = "策略唯一ID", example = 42)
    ),
    responses(
    ( status=200, description="删除成功"),
    ( status=404, description="策略不存在"),
    ( status=403, description="删除错误"),
    ),
    tag = CEDAR_POLICY_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn delete_policy(
    State(service): State<CedarPolicyService>,
    Path(policy_id): Path<i32>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    service.delete_policy(
        current_user,
        context,
        policy_id,
    ).await?;
    
    Ok(StatusCode::NO_CONTENT)
}


#[utoipa::path(
    post,
    path = "/cache",
    responses(( status=202, description = "更新成功"),
                (status=404, description="策略不存在"),),
    tag = CEDAR_POLICY_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn update_policies_cache(
    State(service): State<CedarPolicyService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
     service.update_policies_cache(
        current_user,
        context
    ).await?;
    Ok(StatusCode::ACCEPTED)
}