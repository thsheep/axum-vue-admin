use serde::{Deserialize, Serialize};
use sea_orm::FromQueryResult;
use utoipa::{IntoParams, ToSchema};
use chrono::{DateTime, Utc};
use validator::Validate;
use crate::utils::function::{default_page, default_page_size, default_true};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CedarContext {
    pub source_ip: String,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub authn_mfa: bool,
    // pub time: u64 
}

#[derive(Debug, Deserialize, IntoParams, Validate, Clone)]
#[allow(dead_code)]
pub struct QueryParams {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_page_size", alias = "pageSize")]
    pub page_size: u64,
    pub effect: Option<String>,
    pub is_active: Option<bool>,
    pub fields: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreatePolicyDto {
    pub policy_text: String,
    #[serde(default="default_true")]
    pub is_active: bool,
    pub description: String,
}



#[derive(Default, Debug, Serialize, Deserialize, FromQueryResult, ToSchema)]
pub struct CedarPolicyResponse {
    #[serde(skip_serializing_if = "Option::is_none", rename = "id")]
    pub policy_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_str_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_user: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


#[derive(Default, Debug, Serialize, Deserialize, FromQueryResult, ToSchema, Validate)]
pub struct UpdateSchema{
    pub schema: String,
    pub description: String,
}

#[derive(Default, Debug, Serialize, Deserialize, FromQueryResult, ToSchema)]
pub struct CedarSchemaResponse{
    pub id: i32,
    pub schema: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}