use crate::config::state::AppState;
use crate::handlers::password;
use crate::services::password::PasswordService;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub fn public_routes(app_state: AppState) -> OpenApiRouter {
    let service = PasswordService::new(app_state);

    OpenApiRouter::new()
        .routes(routes!(password::forgot_password))
        .routes(routes!(password::resets_password))
        .with_state(service)
}
