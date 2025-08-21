use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

pub const USER_TAG: &str = "User";
pub const AUTH_TAG: &str = "Auth";
pub const PASSWORD_TAG: &str = "Password";
pub const ROLE_TAG: &str = "Role";
pub const DEPARTMENT_TAG: &str = "Department";
pub const ME_TAG: &str = "Me";
pub const GROUP_TAG: &str = "Group";

pub const CEDAR_POLICY_TAG: &str = "Cedar Policy";

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = USER_TAG, description = "User API endpoints"),
        (name = AUTH_TAG, description = "Auth API endpoints"),
        (name = PASSWORD_TAG, description = "Password API endpoints"),
        (name = ROLE_TAG, description = "User Role API endpoints"),
        (name = DEPARTMENT_TAG, description = "Department API endpoints"),
        (name = ME_TAG, description = "User Profile API endpoints"),
        (name = CEDAR_POLICY_TAG, description = "Cedar Policy API endpoints"),
    ),
    modifiers(&SecurityAddon),
    security(
        // 全局安全要求 - 所有端点都需要认证
        // ("bearerAuth" = [])
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearerAuth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        )
    }
}
