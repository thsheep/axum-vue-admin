// 路由模块入口

use axum::middleware;
use crate::config::state::AppState;
use utoipa_axum::router::OpenApiRouter;
use crate::middlewares::auth_guard::auth_guard_middleware;

mod audit_log;
mod auth;
mod department;
mod health;
mod me;
mod password;
mod role;
mod user;
mod group;
mod sse;
mod cedar_policy;
mod cedar_schema;

pub fn public_router(app_state: AppState) -> OpenApiRouter {
    let routes = OpenApiRouter::new()
        .nest("/auth", auth::public_routes(app_state.clone()))
        .nest("/password-resets", password::public_routes(app_state.clone()));
    
    routes
}



pub fn protected_router(app_state: AppState) -> OpenApiRouter {

    let routes = OpenApiRouter::new()
        .nest("/auth", auth::protected_routes(app_state.clone()))
        .nest("/users", user::protected_routes(app_state.clone()))
        .nest("/roles", role::protected_routes(app_state.clone()))
        .nest(
            "/departments",
            department::protected_routes(app_state.clone()),
        )
        .nest("/groups", group::protected_routes(app_state.clone()))
        .nest("/me", me::protected_routes(app_state.clone()))
        .nest("/cedar_policies", cedar_policy::protected_routes(app_state.clone()))
        .nest("/cedar_schema", cedar_schema::protected_routes(app_state.clone()))
        .nest("/event", sse::protected_routes(app_state.clone()))
        .layer(middleware::from_fn_with_state(
            app_state.clone(), auth_guard_middleware
        ));
    
    routes
}
