use serde_with::DisplayFromStr;
use cedar_policy::{EntityUid, PolicyId};
use serde::{Deserialize, Serialize};
use sea_orm::FromQueryResult;
use utoipa::{IntoParams, ToSchema};
use chrono::{DateTime, Utc};
use serde_with::serde_as;
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
    pub policy_type: String,
    #[serde(default="default_true")]
    pub is_active: bool,
    pub description: String,
}



#[derive(Default, Debug, Serialize, Deserialize, FromQueryResult, ToSchema)]
pub struct CedarPolicyResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_type: Option<String>,
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
    pub uuid: String,
    pub schema: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


/// 一持久化的模板链接信息。
#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct TemplateLinkRecord {
    #[serde_as(as = "DisplayFromStr")]
    pub link_uuid: PolicyId,
    #[serde_as(as = "DisplayFromStr")]
    pub template_uuid: PolicyId,
    #[serde_as(as = "DisplayFromStr")]
    pub principal_uid: EntityUid,
    #[serde_as(as = "DisplayFromStr")]
    pub resource_uid: EntityUid,
}