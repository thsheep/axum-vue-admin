use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct CreateDepartmentDto {
    pub name: String,
    pub desc: String,
    pub order: i32,
    pub parent_uuid: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct DeptTreeNode {
    pub uuid: String,
    pub name: String,
    pub desc: Option<String>,
    pub order: i32,
    pub parent_uuid: String,
    #[schema(no_recursion)]
    pub children: Vec<DeptTreeNode>, // 子部门
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DepartmentResponse {
    pub uuid: String,
    pub name: String,
    pub desc: Option<String>,
    pub order: i32,
    pub parent_uuid: String,
}

#[derive(Default, Debug, Serialize, Deserialize, ToSchema, FromQueryResult)]
pub struct DeptUserResponse {
    pub uuid: String,
    pub name: String,
}