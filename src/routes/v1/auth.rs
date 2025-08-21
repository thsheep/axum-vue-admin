// 认证相关路由（登录、SSO等）

use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::{handlers::auth};
use crate::config::state::AppState;
use crate::services::auth::AuthService;


pub fn public_routes(app_state: AppState) -> OpenApiRouter {
    let service = AuthService::new(app_state);
    OpenApiRouter::new()
        .routes(routes!(auth::login))
        .routes(routes!(auth::refresh_token))
        .with_state(service)
}

pub fn protected_routes(app_state: AppState) -> OpenApiRouter {
    let service = AuthService::new(app_state);
    OpenApiRouter::new()
        .routes(routes!(auth::logout))
        .with_state(service)
}