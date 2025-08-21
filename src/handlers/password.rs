use crate::config::openapi::PASSWORD_TAG;
use crate::errors::app_error::AppError;
use crate::schemas::password::{ForgotPasswordDto, ResetPasswordDto};
use crate::services::password::PasswordService;
use axum::{Json, extract::{State, Path},
           http::StatusCode
};
use axum::response::IntoResponse;
use axum_extra::TypedHeader;
use headers::Referer;
use validator::Validate;
use crate::bad_request;

#[utoipa::path(
    post,
    path = "",
    request_body=ForgotPasswordDto,
    responses(
    ( status=201, description = "重置邮件发送成功"),
    ),
    tag = PASSWORD_TAG
)]
pub async fn forgot_password(
    State(service): State<PasswordService>,
    referer: Option<TypedHeader<Referer>>,
    Json(dto): Json<ForgotPasswordDto>,
) -> Result<impl IntoResponse, AppError> {
    dto.validate()?;

    let referer_url = match referer {
        Some(referer_url) => referer_url.to_string(),
        None => {
            return Err(bad_request!("Bad Request"))
        }
    };
    service.forgot_password(referer_url, dto).await?;
    Ok(StatusCode::ACCEPTED)
}


#[utoipa::path(
    post,
    path = "/{reset_token}",
    request_body=ResetPasswordDto,
    responses(
    ( status=204, description = "密码重置成功"),
    ),
    tag = PASSWORD_TAG
)]
pub async fn resets_password(
    State(service): State<PasswordService>,
    Path(reset_token): Path<String>,
    Json(dto): Json<ResetPasswordDto>,
) -> Result<impl IntoResponse, AppError> {
    dto.validate()?;

    service.resets_password(reset_token, dto).await?;
    Ok(StatusCode::NO_CONTENT)
}