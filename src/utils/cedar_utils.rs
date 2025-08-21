use crate::errors::app_error::AppError;
use crate::schemas::auth::CurrentUser;
use crate::schemas::cedar_policy::CedarContext;
use cedar_policy::{Context, Entities, EntityUid, Request};
use serde_json::{json};
use std::str::FromStr;
use tracing::log::debug;
use crate::forbidden;


static APPLICATION_ENTITY_UID: &str = r#"Application::"VueAxumAdmin""#;

/// 授权操作定义
#[derive(Debug, Clone, Copy)]
pub enum AuthAction {
    ViewUser,
    CreateUser,
    UpdateUser,
    DeleteUser,
    ViewDepartment,
    ViewDepartmentUsers,
    CreateDepartment,
    UpdateDepartment,
    MoveDepartment,
    DeleteDepartment,
    ViewGroup,
    ViewGroupUsers,
    CreateGroup,
    UpdateGroup,
    DeleteGroup,
    ViewRole,
    CreateRole,
    UpdateRole,
    DeleteRole,
    AssignRole,
    RevokeRole,
    ViewAuditLog,
    ViewPolicy,
    CreatePolicy,
    UpdatePolicy,
    DeletePolicy,
}

impl AuthAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthAction::ViewUser => r#"Action::"ViewUser""#,
            AuthAction::CreateUser => r#"Action::"CreateUser""#,
            AuthAction::UpdateUser => r#"Action::"UpdateUser""#,
            AuthAction::DeleteUser => r#"Action::"DeleteUser""#,
            AuthAction::ViewDepartment => r#"Action::"ViewDepartment""#,
            AuthAction::ViewDepartmentUsers => r#"Action::"ViewDepartmentUsers""#,
            AuthAction::CreateDepartment => r#"Action::"CreateDepartment""#,
            AuthAction::UpdateDepartment => r#"Action::"UpdateDepartment""#,
            AuthAction::MoveDepartment => r#"Action::"MoveDepartment""#,
            AuthAction::DeleteDepartment => r#"Action::"DeleteDepartment""#,
            AuthAction::ViewGroup => r#"Action::"ViewGroup""#,
            AuthAction::ViewGroupUsers => r#"Action::"ViewGroupUsers""#,
            AuthAction::CreateGroup => r#"Action::"CreateGroup""#,
            AuthAction::UpdateGroup => r#"Action::"UpdateGroup""#,
            AuthAction::DeleteGroup => r#"Action::"DeleteGroup""#,
            AuthAction::ViewRole => r#"Action::"ViewRole""#,
            AuthAction::CreateRole => r#"Action::"CreateRole""#,
            AuthAction::UpdateRole => r#"Action::"UpdateRole""#,
            AuthAction::DeleteRole => r#"Action::"DeleteRole""#,
            AuthAction::AssignRole => r#"Action::"AssignRole""#,
            AuthAction::RevokeRole => r#"Action::"RevokeRole""#,
            AuthAction::ViewAuditLog => r#"Action::"ViewAuditLog""#,
            AuthAction::ViewPolicy => r#"Action::"ViewPolicies""#,
            AuthAction::CreatePolicy => r#"Action::"CreatePolicies""#,
            AuthAction::UpdatePolicy => r#"Action::"UpdatePolicies""#,
            AuthAction::DeletePolicy => r#"Action::"DeletePolicies""#,
        }
    }
}

/// 资源类型定义
#[derive(Debug, Clone)]
pub enum ResourceType {
    User(Option<i32>),      // User::* 或 User::{id}
    Department(Option<i32>), // Department::* 或 Department::{id}
    Group(Option<i32>),     // Group::* 或 Group::{id}
    Role(Option<i32>),      // Role::* 或 Role::{id}
    Policy(Option<i32>), // CedarPolicy::*
    UI(Option<String>),
    AuditLog,               // AuditLog::*
}

impl ResourceType {
    pub fn as_entity_uid(&self) -> Result<EntityUid, AppError> {
        let uid_str = match self {
            ResourceType::User(Some(user_id)) => format!(r#"User::"{}""#, user_id),
            ResourceType::User(None) => r#"User::"*""#.to_string(),
            ResourceType::Department(Some(dept_id)) => format!(r#"Department::"{}""#, dept_id),
            ResourceType::Department(None) => r#"Department::"*""#.to_string(),
            ResourceType::Group(Some(group_id)) => format!(r#"Group::"{}""#, group_id),
            ResourceType::Group(None) => r#"Group::"*""#.to_string(),
            ResourceType::Role(Some(role_id)) => format!(r#"Role::"{}""#, role_id),
            ResourceType::Role(None) => r#"Role::"*""#.to_string(),
            ResourceType::Policy(Some(id)) => format!(r#"Policy::"{}""#, id),
            ResourceType::Policy(None) => r#"Policy::"*""#.to_string(),
            ResourceType::AuditLog => r#"AuditLog::"*""#.to_string(),
            ResourceType::UI(Some(uid)) => format!(r#"UI::"{}""#, uid),
            &ResourceType::UI(None) => r#"UI::"*""#.to_string()
        };

        EntityUid::from_str(&uid_str).map_err(|e| forbidden!(format!("Wrong entity UID: {}", e)))
    }
}

/// 授权检查构建器
pub struct AuthorizationBuilder {
    current_user: CurrentUser,
    context: CedarContext,
    action: AuthAction,
    resource: ResourceType,
    resource_entities: Entities,
}

impl AuthorizationBuilder {
    pub fn new(current_user: CurrentUser, context: CedarContext) -> Self {
        Self {
            current_user,
            context,
            action: AuthAction::ViewUser, // 默认值
            resource: ResourceType::User(None), // 默认值
            resource_entities: Entities::empty(),
        }
    }

    pub fn action(mut self, action: AuthAction) -> Self {
        self.action = action;
        self
    }

    pub fn resource(mut self, resource: ResourceType) -> Self {
        self.resource = resource;
        self
    }

    pub fn resource_entities(mut self, entities: Entities) -> Self {
        self.resource_entities = entities;
        self
    }

    pub fn build(self) -> Result<(Request, String, Entities), AppError> {
        let principal_str = format!(r#"User::"{}""#, self.current_user.user_id);
        let principal = EntityUid::from_str(principal_str.as_str())?;
        let action = EntityUid::from_str(self.action.as_str())?;
        let resource = self.resource.as_entity_uid()?;
        let context = Context::from_json_value(json!(self.context), None)?;

        let request = Request::new(principal, action, resource, context, None)?;
        debug!("Request: {:?}", request);

        Ok((request, self.current_user.username, self.resource_entities))
    }
}



pub fn entities2json(entities: &Entities) -> Result<String, AppError> {
    let mut buffer = Vec::new();
    entities.write_to_json(&mut buffer)?;
    let entities_json_str =String::from_utf8(buffer).map_err(anyhow::Error::from)?;
    Ok(entities_json_str)
}