// 角色管理路由

use crate::config::state::AppState;
use crate::handlers::role;
use crate::services::role::RoleService;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub fn protected_routes(app_state: AppState) -> OpenApiRouter {
    let service = RoleService::new(app_state);

    OpenApiRouter::new()
        .routes(routes!(role::list_roles, role::create_role))
        .routes(routes!(
            role::get_role,
            role::update_role,
            role::delete_role
        ))
        .with_state(service)
}
