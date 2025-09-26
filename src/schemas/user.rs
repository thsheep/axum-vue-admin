use chrono::{NaiveDateTime, Utc};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::entity::users::Model as UserModel;

// 定义个userid的别名类型，万一后面切换为uuid呢...
pub type UserUUID = String;

fn default_page() -> u64 {
    1
}

fn default_page_size() -> u64 {
    10
}

#[derive(Debug, Deserialize, IntoParams, Validate, Clone)]
#[allow(dead_code)]
pub struct QueryParams {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_page_size", alias = "pageSize")]
    pub page_size: u64,
    pub username: Option<String>,
    pub email: Option<String>,
    pub dept_uuid: Option<String>,
    pub fields: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct AssignRoleDto {
    pub role_uuid: String,
}

fn default_true() -> bool {
    true
}
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateUserDto {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 3, max = 100))]
    pub username: String,
    #[validate(length(min = 6))]
    pub password: String,
    pub groups: Vec<String>,
    pub dept: String,
    pub alias: Option<String>,
    pub phone: Option<String>,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateUserDto {
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 3, max = 100))]
    pub username: Option<String>,
    pub groups: Option<Vec<String>>,
    pub dept: Option<String>,
    pub alias: Option<String>,
    pub phone: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromQueryResult)]
pub struct DeptResponse {
    pub uuid: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromQueryResult)]
pub struct GroupResponse {
    pub uuid: String,
    pub name: String,
}
#[derive(Default, Debug, Serialize, Deserialize, ToSchema)]
pub struct UserResponse {
    pub uuid: String,
    pub username: String,
    pub alias: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub is_active: bool,
    pub dept: Option<DeptResponse>,
    pub groups: Vec<GroupResponse>,
    pub avatar: Option<String>,
    pub last_login: Option<NaiveDateTime>,
}

impl From<UserModel> for UserResponse {
    fn from(user: UserModel) -> Self {
        Self {
            uuid: user.user_uuid,
            username: user.username,
            alias: user.alias,
            email: user.email,
            phone: user.phone,
            is_active: user.is_active,
            avatar: user.avatar,
            last_login: user.last_login,
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, FromQueryResult)]
pub struct UserRoleResponse {
    uuid: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, FromQueryResult)]
pub struct UserDeptResponse {
    uuid: String,
    name: String,
}



// 用户角色信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRoleInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    pub role_name: String,
    pub source: String,      // "direct" 或 "group"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>, // 如果来源是组，则包含组名
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectRole {
    pub uuid: String,
    pub role_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRole {
    pub uuid: String,
    pub role_name: String,
    pub group_name: String,
}