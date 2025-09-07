use crate::entity::roles::Model as RoleModel;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use chrono::{DateTime, Utc};
use validator::Validate;

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
    pub name: Option<String>,
    pub fields: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateRoleDto {
    #[validate(length(min = 3, max = 100))]
    pub name: String,
    pub description: String,
}


#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateRoleDto {
    pub id: i32,
    #[validate(length(min = 3, max = 100))]
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromQueryResult, ToSchema)]
pub struct RoleFieldResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}


#[derive(Debug, Serialize, Deserialize, ToSchema, FromQueryResult)]
pub struct RoleResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

impl From<RoleModel> for RoleResponse {
    fn from(role: RoleModel) -> Self {
        Self {
            id: Some(role.role_id),
            created_at: Some(role.created_at),
            name: Some(role.role_name),
            description: role.description,
        }
    }
}


#[derive(Debug, Serialize, Deserialize, ToSchema, FromQueryResult)]
pub struct RolePermissionResponse {
    pub id: i64,
    pub name: String,
    pub slug: String,
}

