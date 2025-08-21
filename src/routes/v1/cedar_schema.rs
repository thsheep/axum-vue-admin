use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use crate::config::state::AppState;
use crate::services::cedar_schema::CedarSchemaService;
use crate::handlers::cedar_schema;

pub fn protected_routes(app_state: AppState) -> OpenApiRouter {
    let service = CedarSchemaService::new(app_state);
    OpenApiRouter::new()
        .routes(routes!(cedar_schema::list_schema, cedar_schema::update_schema))
        .with_state(service)
}