use crate::services::sse::SSEService;
use axum::routing::any;
use utoipa_axum::router::OpenApiRouter;

use crate::handlers::sse;
use crate::config::state::AppState;

pub fn protected_routes(app_state: AppState) -> OpenApiRouter {
    let service = SSEService::new(app_state);

    OpenApiRouter::new()
        .route("/global_message", any(sse::global_message_push))
        .with_state(service)
}