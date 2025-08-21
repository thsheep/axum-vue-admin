use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;
use validator::Validate;

// 用户认证信息
#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub user_id: i32,
    pub username: String,
    pub is_super_admin: bool,
}


#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct PermissionCheckDto {
    pub user_id: i32,
    pub permission: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthResponse {
    pub access_token: String,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub iat: u64,
    pub exp: u64,
    pub jti: Uuid,
    pub name: String,
    pub token_type: TokenType,
    pub is_super_admin: bool,
}
