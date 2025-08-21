use axum::extract::Query;
use axum::{
    extract::{Path, State, Extension},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use validator::Validate;

use crate::schemas::paginated::PaginatedApiResponse;
use crate::schemas::response::ApiResponse;
use crate::schemas::user::{
    AssignRoleDto, CreateUserDto, QueryParams, UpdateUserDto,
    UserResponse, UserRoleResponse,
};
use crate::{
    config::openapi::USER_TAG,
    errors::app_error::AppError,
    services::user::UserService,
};
use crate::schemas::auth::CurrentUser;
use crate::schemas::cedar_policy::CedarContext;

#[utoipa::path(get, path = "",
    params(QueryParams),
    responses((status = 200, body = Vec<UserResponse>),),
    tag = USER_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn list_users(
    State(service): State<UserService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
    Query(params): Query<QueryParams>,
) -> Result<impl IntoResponse, AppError> {
    params.validate()?;
    let (users, total) = service.list_users(
        current_user,
        context,
        params.clone()
    ).await?;
    Ok(PaginatedApiResponse::success(
        users,
        total,
        params.page,
        params.page_size,
        StatusCode::OK,
    ))
}

#[utoipa::path(
    get,
    path = "/{user_id}",  // 路径中的 {id} 会被识别为参数
    params(
        ("user_id" = u64, Path, description = "用户唯一ID", example = 42)
    ),
    responses(
        (status = 200, description = "用户详情", body = UserResponse),
        (status = 404, description = "用户不存在"),
    ),
    tag = USER_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn get_user(
    Path(user_id): Path<i32>,
    State(service): State<UserService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    let user = service.get_user(
        current_user,
        context,
        user_id).await?;
    Ok(ApiResponse::success(user, StatusCode::OK))
}

#[utoipa::path(
    post,
    path = "",
    request_body=CreateUserDto,
    responses(( status=201, body=UserResponse, description = "创建用户成功"),
                (status=409, description = "用户名或邮箱已存在"),),
    tag = USER_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn create_user(
    State(service): State<UserService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
    Json(dto): Json<CreateUserDto>,
) -> Result<impl IntoResponse, AppError> {
    dto.validate()?;
    let user = service.create_user(
        current_user,
        context,
        dto).await?;
    Ok(ApiResponse::success(user, StatusCode::CREATED))
}

#[utoipa::path(
    put,
    path = "/{user_id}",
    request_body=UpdateUserDto,
    params(
        ("user_id" = u64, Path, description = "用户唯一ID", example = 42)
    ),
    responses(  (status=200, body=UserResponse, description="更新成功"),
                (status=404, description="用户不存在"),),
    tag = USER_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn update_user(
    Path(user_id): Path<i32>,
    State(service): State<UserService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
    Json(dto): Json<UpdateUserDto>,
) -> Result<impl IntoResponse, AppError> {
    dto.validate()?;
    let user = service.update_user(
        current_user,
        context,
        user_id, dto).await?;
    Ok(ApiResponse::success(user, StatusCode::OK))
}

#[utoipa::path(
    delete,
    path = "/{user_id}",
    params(
        ("user_id" = u64, Path, description = "用户唯一ID", example = 42)
    ),
    responses(( status=200, description="删除成功"),),
    tag = USER_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn delete_user(
    Path(user_id): Path<i32>,
    State(service): State<UserService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    service.delete_user(
        current_user,
        context,
        user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/{user_id}/roles",
    params(
        ("user_id" = u64, Path, description = "用户唯一ID", example = 42)
    ),
    responses(( status=200, description="用户角色列表", body=Vec<UserRoleResponse>),),
    tag = USER_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn user_roles(
    Path(user_id): Path<i32>,
    State(service): State<UserService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    let roles = service.user_roles(
        current_user,
        context,
        user_id).await?;
    Ok(ApiResponse::success(roles, StatusCode::OK))
}

#[utoipa::path(
    post,
    path = "/{user_id}/roles",
    request_body=AssignRoleDto,
    params(
        ("user_id" = i32, Path, description = "用户唯一ID", example = 42)
    ),
    responses(( status=201, description="分配角色成功"),),
    tag = USER_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn assign_roles(
    Path(user_id): Path<i32>,
    State(service): State<UserService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
    Json(dto): Json<AssignRoleDto>, // Json提取器需要放在最后，负责会报错。
) -> Result<impl IntoResponse, AppError> {
    service.assign_roles(
        current_user,
        context,
        user_id, dto).await?;
    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    delete,
    path = "/{user_id}/roles/{role_id}",
    params(
        ("user_id" = i32, Path, description = "用户唯一ID", example = 42),
        ("role_id" = i32, Path, description = "角色唯一ID", example = 42)
    ),
    responses(( status=204, description="移除用户角色成功"),),
    tag = USER_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn revoke_roles(
    Path((user_id, role_id)): Path<(i32, i32)>,
    State(service): State<UserService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    service.revoke_roles(
        current_user,
        context,
        user_id, role_id).await?;
    Ok(StatusCode::NO_CONTENT)
}