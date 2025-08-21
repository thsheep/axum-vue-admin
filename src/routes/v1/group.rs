use crate::config::state::AppState;
use crate::handlers::group;
use crate::services::groups::GroupService;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;


pub fn protected_routes(app_state: AppState) -> OpenApiRouter {
    let service = GroupService::new(app_state);
    OpenApiRouter::new()
        .routes(routes!(group::list_groups, group::create_group))
        .routes(routes!(group::get_group, group::update_group, group::delete_group))
        .routes(routes!(group::assign_users, group::revoke_users))
        .routes(routes!(group::assign_roles, group::get_group_roles, group::revoke_roles))
        .with_state(service)
}