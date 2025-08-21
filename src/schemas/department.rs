use sea_orm::FromQueryResult;
use crate::entity::departments::Model as DepartmentModel;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct CreateDepartmentDto {
    pub name: String,
    pub desc: String,
    pub order: i32,
    pub parent_id: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct DeptTreeNode {
    pub id: i32,
    pub name: String,
    pub desc: Option<String>,
    pub order: i32,
    pub parent_id: i32,
    #[schema(no_recursion)]
    pub children: Vec<DeptTreeNode>, // 子部门
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DepartmentResponse {
    pub id: i32,
    pub name: String,
    pub desc: Option<String>,
    pub order: i32,
    pub parent_id: i32,
}

impl From<DepartmentModel> for DepartmentResponse {
    fn from(entity: DepartmentModel) -> Self {
        Self {
            id: entity.dept_id,
            name: entity.name,
            desc: entity.desc,
            order: entity.order,
            parent_id: entity.parent_id,
        }
    }
}


#[derive(Default, Debug, Serialize, Deserialize, ToSchema, FromQueryResult)]
pub struct DeptUserResponse {
    pub id: i32,
    pub name: String,
}