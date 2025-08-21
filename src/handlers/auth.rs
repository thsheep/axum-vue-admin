// 认证相关路由（登录、SSO等）

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use axum_extra::TypedHeader;
use validator::Validate;

use crate::schemas::response::ApiResponse;
use crate::{
    config::openapi::AUTH_TAG,
    errors::app_error::AppError,
    services::auth::AuthService,
};
use crate::schemas::auth::{AuthResponse, Credentials};

#[utoipa::path(
    post,
    path = "/login",
    request_body=Credentials,
    responses(( status=200, body=AuthResponse, description = "登陆成功"),
                (status=401, description = "认证失败"),),
    tag = AUTH_TAG
)]
pub async fn login(
    State(service): State<AuthService>,
    jar: CookieJar,
    Json(dto): Json<Credentials>,
) -> Result<(CookieJar, ApiResponse<AuthResponse>), AppError> {
    dto.validate()?;
    let (cookie_jar, auth_response) = service.authenticate(jar, dto).await?;
    Ok((cookie_jar, ApiResponse::success(auth_response, StatusCode::OK)))
}


#[utoipa::path(
    post,
    path = "/refresh_token",
    responses(( status=200, body=AuthResponse, description = "刷新成功")),
    tag = AUTH_TAG
)]
pub async fn refresh_token(
    State(service): State<AuthService>,
    jar: CookieJar,
) -> Result<(CookieJar, ApiResponse<AuthResponse>), AppError> {
    let (cookie_jar, auth_response) = service.refresh(jar).await?;
    Ok((cookie_jar, ApiResponse::success(auth_response, StatusCode::OK)))
}


#[utoipa::path(
    post,
    path = "/logout",
    responses(( status=200, description = "退出成功")),
    tag = AUTH_TAG,
    security(
          ("bearerAuth" = [])
        ),
)]
pub async fn logout(
    State(service): State<AuthService>,
    jar: CookieJar,
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
) -> Result<(CookieJar, impl IntoResponse), AppError> {
    let cookie_jar = service.logout(jar, auth_header).await?;
    Ok((cookie_jar, StatusCode::OK))
}