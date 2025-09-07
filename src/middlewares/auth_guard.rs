use crate::config::app::BLACK_LIST_JTI;
use crate::config::state::AppState;
use crate::errors::app_error::AppError;
use crate::schemas::{auth::CurrentUser, cedar_policy::CedarContext};
use crate::utils::jwt::decode_token;
use axum::{
    extract::{ConnectInfo, Request, State},
    middleware::Next,
    response::Response,
};
use redis::AsyncCommands;
use std::net::SocketAddr;
use std::str::FromStr;
use cedar_policy::{Context, Expression};
use crate::unauthorized;

pub async fn auth_guard_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    if let Some(token) = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        let payload = decode_token(token)?;
        // 检查黑名单...
        let mut redis_conn = state.redis.get_multiplexed_async_connection().await?;
        if redis_conn
            .exists::<_, bool>(format!("{}:{}", BLACK_LIST_JTI, payload.jti))
            .await?
        {
            return Err(unauthorized!("InvalidToken".to_string()));
        };

        let current_user = CurrentUser {
            user_id: payload.sub,
            dept_id: payload.dept_id,
            username: payload.name,
            is_super_admin: payload.is_super_admin,
        };
        req.extensions_mut().insert(current_user);

        let remote_addr = req
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ci| ci.0)
            .unwrap_or(SocketAddr::from_str("127.0.0.1:6000").unwrap())
            .ip()
            .to_string();
        
        let cedar_context = CedarContext {
            source_ip: remote_addr
        };
        req.extensions_mut().insert(cedar_context);

        return Ok(next.run(req).await);
    };
    return Err(unauthorized!("Unauthorized".to_string()));
}
