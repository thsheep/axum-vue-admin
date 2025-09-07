use chrono::NaiveDateTime;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::entity::users::Model as UserModel;

// 定义个userid的别名类型，万一后面切换为uuid呢...
pub type UserID = i32;

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
    pub dept_id: Option<u64>,
    pub fields: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct AssignRoleDto {
    pub ids: Vec<i32>,
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
    pub groups: Vec<i32>,
    pub dept: i32,
    pub alias: Option<String>,
    pub phone: Option<String>,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateUserDto {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 3, max = 100))]
    pub username: String,
    pub groups: Vec<i32>,
    pub dept: i32,
    pub alias: Option<String>,
    pub phone: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromQueryResult)]
pub struct DeptResponse {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromQueryResult)]
pub struct GroupResponse {
    pub id: i32,
    pub name: String,
}
#[derive(Default, Debug, Serialize, Deserialize, ToSchema)]
pub struct UserResponse {
    pub id: i32,
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
            id: user.user_id,
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
    id: i32,
    name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, FromQueryResult)]
pub struct UserDeptResponse {
    id: i32,
    name: String,
}



// 用户角色信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRoleInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub role_name: String,
    pub source: String,      // "direct" 或 "group"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>, // 如果来源是组，则包含组名
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectRole {
    pub id: i32,
    pub role_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRole {
    pub id: i32,
    pub role_name: String,
    pub group_name: String,
}

// 用户权限信息结构
// 响应数据结构
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserPermissionItem {
    pub source_type: String,           // "direct_role", "group_inherited", "role_inherited"
    pub source_id: i32,               // 角色ID或用户组ID
    pub source_name: String,          // 角色名或用户组名
    pub permission_id: i32,           // 权限ID
    pub permission_name: String,      // 权限名称
    pub permission_slug: String,      // 权限标识符
    pub module: String,               // 所属模块
    pub role_id: Option<i32>,         // 如果是用户组继承，记录具体的角色ID
    pub role_name: Option<String>,    // 如果是用户组继承，记录具体的角色名
    pub inherited_from_role_id: Option<i32>, // 如果是角色继承，记录父角色ID
    pub inherited_from_role_name: Option<String>, // 如果是角色继承，记录父角色名
}