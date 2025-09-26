use chrono::{DateTime, Utc};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;
use crate::entity::user_groups::Model as GroupModel;

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
    #[serde(default = "default_page_size")]
    pub page_size: u64,
    pub name: Option<String>,
    pub fields: Option<String>,
}


#[derive(Default, Debug, Serialize, Deserialize, FromQueryResult, ToSchema)]
pub struct GroupResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<GroupModel> for GroupResponse {
    fn from(group: GroupModel) -> Self {
        Self {
            uuid: Some(group.user_group_uuid),
            name: Some(group.name),
            description: group.description,
            created_at: Some(group.created_at),
            updated_at: Some(group.updated_at),
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateGroupDto {
    #[validate(length(min = 3, max = 100))]
    pub name: String,
    #[validate(length(min = 0, max = 255))]
    pub description: Option<String>,
}


#[derive(Default, Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct AssignUsersDto {
    pub user_uuids: Vec<String>,
}

#[derive(Default, Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct AssignRolesDto {
    pub role_uuid: String,
}


#[derive(Default, Debug, Serialize, Deserialize, ToSchema, FromQueryResult, Validate)]
pub struct GroupRoleResponse {
    pub uuid: String,
    pub name: String,
}
