use crate::schemas::user::{DeptResponse, GroupResponse};
use chrono::NaiveDateTime;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use utoipa::ToSchema;

// 定义UI策略的类型别名，便于理解
pub type UiPolicies = HashSet<String>;


#[derive(Default, Debug, Serialize, Deserialize, ToSchema, FromQueryResult)]
pub struct Info {
    #[serde(rename = "uuid")]
    pub user_uuid: String,
    pub username: String,
    pub alias: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub is_active: bool,
    pub avatar: Option<String>,
    pub last_login: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Profile {
    pub ui_policies: UiPolicies,
    pub roles: Vec<String>,
    pub info: Info,
    pub departments: Option<DeptResponse>,
    pub groups: Vec<GroupResponse>,
}