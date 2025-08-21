use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct ForgotPasswordDto {
    #[validate(email)]
    pub email: String,
    pub language: String,
}


#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct ResetPasswordDto {
    #[serde(alias = "newPassword")]
    pub new_password: String,
}