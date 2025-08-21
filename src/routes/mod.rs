use utoipa_axum::router::OpenApiRouter;
use crate::config::state::AppState;

mod v1;


pub fn api_router(app_state: AppState) -> OpenApiRouter {
    OpenApiRouter::new()
    .nest("/v1", v1::protected_router(app_state.clone()))
        .nest("/v1", v1::public_router(app_state.clone()))
}