use crate::config::state::AppState;
use crate::handlers::me;
use crate::services::me::MeService;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub fn protected_routes(app_state: AppState) -> OpenApiRouter {
    let service = MeService::new(app_state);
    OpenApiRouter::new()
        .routes(routes!(me::profile))
        .with_state(service)
}
