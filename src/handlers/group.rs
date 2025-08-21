use crate::config::openapi::GROUP_TAG;
use crate::schemas::{
    groups::{CreateGroupDto, AssignUsersDto, GroupResponse, QueryParams, GroupRoleResponse,
             AssignRolesDto},
    paginated::PaginatedApiResponse,
    response::ApiResponse,
};
use crate::{errors::app_error::AppError, services::groups::GroupService};
use axum::{extract::{Json, Path, Query, State}, http::StatusCode, response::IntoResponse, Extension};
use validator::Validate;
use crate::handlers::group;
use crate::schemas::auth::CurrentUser;
use crate::schemas::cedar_policy::CedarContext;

#[utoipa::path(
    get,
    path = "",
    params(QueryParams),
    responses((status = 200, body = Vec<GroupResponse>),),
    tag = GROUP_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn list_groups(
    State(service): State<GroupService>,
    Query(params): Query<QueryParams>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    params.validate()?;
    let (groups, total) = service.list_groups(
        current_user,
        context,
        params.clone()).await?;
    Ok(PaginatedApiResponse::success(groups,
                                     total,
                                     params.page,
                                     params.page_size,
                                     StatusCode::OK
    ))
}

#[utoipa::path(
    post,
    path = "",
    request_body=CreateGroupDto,
    responses(( status=200, body=GroupResponse,  description = "创建用户组成功"),
                (status=409, description = "用户组已存在"),),
    tag = GROUP_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn create_group(
    State(service): State<GroupService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
    Json(payload): Json<CreateGroupDto>,
) -> Result<ApiResponse<GroupResponse>, AppError> {
    payload.validate()?;
    let group = service.create_group(
        current_user,
        context,
        payload).await?;
    Ok(ApiResponse::success(group, StatusCode::CREATED))
}

#[utoipa::path(
    get,
    path = "/{group_id}",
    params(
        ("group_id" = i32, Path, description = "用户组唯一ID", example = 42)
    ),
    responses(( status=200, body=GroupResponse, description = "获取成功"),
    ( status=404, description = "不存在"),),
    tag = GROUP_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn get_group(
    Path(group_id): Path<i32>,
    State(service): State<GroupService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    let group = service.get_group(
        current_user,
        context,
        group_id).await?;
    Ok(ApiResponse::success(group, StatusCode::OK))
}

#[utoipa::path(
    put,
    path = "/{group_id}",
    request_body=CreateGroupDto,
    params(
        ("group_id" = i32, Path, description = "用户组唯一ID", example = 42)
    ),
    responses(( status=200, body=GroupResponse, description = "更新成功"),
                (status=404, description="用户组不存在"),),
    tag = GROUP_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn update_group(
    Path(group_id): Path<i32>,
    State(service): State<GroupService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
    Json(dto): Json<CreateGroupDto>,
) -> Result<ApiResponse<GroupResponse>, AppError> {
    let group = service.update_group(
        current_user,
        context,
        group_id, dto).await?;
    Ok(ApiResponse::success(group, StatusCode::OK))
}

#[utoipa::path(
    delete,
    path = "/{group_id}",
    params(
        ("group_id" = i32, Path, description = "用户组唯一ID", example = 42)
    ),
    responses(
    ( status=200, description="删除成功"),
    ( status=404, description="用户组不存在"),
    ( status=403, description="删除错误"),
    ),
    tag = GROUP_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn delete_group(
    Path(group_id): Path<i32>,
    State(service): State<GroupService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
) -> Result<impl IntoResponse, AppError> {
    service.delete_group(
        current_user,
        context,
        group_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/{group_id}/users",
    request_body=AssignUsersDto,
    params(
        ("group_id" = i32, Path, description = "用户组唯一ID", example = 42)
    ),
    responses(( status=201, description="分配用户组成功"),),
    tag = GROUP_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn assign_users(
    Path(group_id): Path<i32>,
    State(service): State<GroupService>,
    Json(dto): Json<AssignUsersDto>,
) -> Result<ApiResponse<()>, AppError> {
    service.assign_users(group_id, dto).await?;
    Ok(ApiResponse::success_empty(StatusCode::OK))
}


#[utoipa::path(
    delete,
    path = "/{group_id}/users/{user_id}",
    request_body=AssignUsersDto,
    params(
        ("group_id" = i32, Path, description = "用户组唯一ID", example = 42),
        ("user_id" = i32, Path, description = "用户唯一ID", example = 42)
    ),
    responses(( status=200, description="删除成功"),),
    tag = GROUP_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn revoke_users(
    Path((group_id, user_id)): Path<(i32, i32)>,
    State(service): State<GroupService>,
) -> Result<impl IntoResponse, AppError> {
    service.revoke_user(group_id, user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}


#[utoipa::path(
    post,
    path = "/{group_id}/roles",
    request_body=AssignRolesDto,
    params(
        ("group_id" = i32, Path, description = "用户组唯一ID", example = 42)
    ),
    responses(( status=201, description="分配角色成功"),),
    tag = GROUP_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn assign_roles(
    Path(group_id): Path<i32>,
    State(service): State<GroupService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>,
    Json(dto): Json<AssignRolesDto>,
) -> Result<impl IntoResponse, AppError> {
    service.assign_roles(
        current_user,
        context,
        group_id,
        dto
    ).await?;
    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    delete,
    path = "/{group_id}/roles/{role_id}",
    params(
        ("group_id" = i32, Path, description = "用户组唯一ID", example = 42),
        ("role_id" = i32, Path, description = "角色唯一ID", example = 42)
    ),
    responses(( status=201, description="移除角色成功"),),
    tag = GROUP_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn revoke_roles(
    Path((group_id, role_id)): Path<(i32, i32)>,
    State(service): State<GroupService>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<CedarContext>
) -> Result<impl IntoResponse, AppError> {
    service.revoke_roles(
        current_user,
        context,
        group_id,
        role_id
    ).await?;
    Ok(StatusCode::NO_CONTENT)
}


#[utoipa::path(
    get,
    path = "/{group_id}/roles",
    params(
        ("group_id" = i32, Path, description = "用户组唯一ID", example = 42)
    ),
    responses(( status=200, body=GroupRoleResponse, description = "获取成功"),
    ( status=404, description = "不存在"),),
    tag = GROUP_TAG,
    security(
      ("bearerAuth" = [])
    ),
)]
pub async fn get_group_roles(
    Path(group_id): Path<i32>,
    State(service): State<GroupService>,
) -> Result<impl IntoResponse, AppError> {
    let group_roles = service.get_group_roles(group_id).await?;
    Ok(ApiResponse::success(group_roles, StatusCode::OK))
}