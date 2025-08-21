use crate::{services::department::DepartmentService};
use crate::config::state::AppState;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::handlers::department;

pub fn protected_routes(app_state: AppState) -> OpenApiRouter {
    let service = DepartmentService::new(app_state);

    OpenApiRouter::new()
        .routes(routes!(
            department::list_departments,
            department::create_department
        ))
        .routes(routes!(
            department::delete_department,
            department::update_department
        ))
        .routes(routes!(department::departments_users))
        .with_state(service)
    // Router::new().route("", get(department::list_departments)).with_state(service)
}
