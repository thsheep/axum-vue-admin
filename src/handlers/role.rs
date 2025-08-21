use axum::extract::Query;
use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Extension, Json};
use validator::Validate;

use crate::schemas::{
    paginated::PaginatedApiResponse,
    response::ApiResponse,
    role::{
        CreateRoleDto, QueryParams, RoleResponse,
        UpdateRoleDto,
    },
};
use crate::{
    config::openapi::ROLE_TAG,
    errors::app_error::AppError,
    services::role::RoleService,
};
use crate::schemas::auth::CurrentUser;
use crate::schemas::cedar_policy::CedarContext;

#[utoipa::path(get, path = "",
    params(QueryParams),
    responses((status = 200, body = Vec<RoleResponse>),),
    tag = ROLE_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn list_roles(
    State(service): State<RoleService>,
    Query(params): Query<QueryParams>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    params.validate()?;
    let (roles, total) = service.list_roles(
        current_user,
        context,
        params.clone()).await?;
    Ok(PaginatedApiResponse::success(
        roles,
        total,
        params.page,
        params.page_size,
        StatusCode::OK,
    ))
}

#[utoipa::path(
    get,
    path = "/{role_id}",  // 路径中的 {id} 会被识别为参数
    params(
        ("role_id" = u64, Path, description = "角色唯一ID", example = 42)
    ),
    responses(
        (status = 200, description = "角色详情", body = RoleResponse),
        (status = 404, description = "角色不存在"),
    ),
    tag = ROLE_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn get_role(
    Path(id): Path<i32>,
    State(service): State<RoleService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    let role = service.get_role(
        current_user,
        context,
        id).await?;
    Ok(ApiResponse::success(role, StatusCode::OK))
}

#[utoipa::path(
    post,
    path = "",
    request_body=CreateRoleDto,
    responses(( status=201, body=RoleResponse, description = "角色创建成功"),
                (status=409, description = "角色已存在"),),
    tag = ROLE_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn create_role(
    State(service): State<RoleService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
    Json(dto): Json<CreateRoleDto>,
) -> Result<impl IntoResponse, AppError> {
    dto.validate()?;
    let role = service.create_role(
        current_user,
        context,
        dto).await?;
    Ok(ApiResponse::success(role, StatusCode::CREATED))
}

#[utoipa::path(
    put,
    path = "/{role_id}",
    request_body=UpdateRoleDto,
    responses(  (status=200, body=RoleResponse, description="更新成功"),
                (status=404, description="角色不存在"),),
    tag = ROLE_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn update_role(
    Path(role_id): Path<i32>,
    State(service): State<RoleService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
    Json(dto): Json<UpdateRoleDto>,
) -> Result<impl IntoResponse, AppError> {
    dto.validate()?;
    let role = service.update_role(
        current_user,
        context,
        role_id,
        dto).await?;
    Ok(ApiResponse::success(role, StatusCode::OK))
}

#[utoipa::path(
    delete,
    path = "/{role_id}",
    params(
        ("role_id" = u64, Path, description = "角色唯一ID", example = 42)
    ),
    responses(( status=204, description="删除成功"),),
    tag = ROLE_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn delete_role(
    Path(role_id): Path<i32>,
    State(service): State<RoleService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    service.delete_role(
        current_user,
        context,
        role_id).await?;
    Ok(StatusCode::NO_CONTENT)
}