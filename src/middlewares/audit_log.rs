use crate::config::state::AppState;
use crate::errors::app_error::AppError;
use crate::schemas::auth::CurrentUser;
use crate::services::audit_log::AuditLogService;
use axum::response::Response;
use axum::{Extension, extract::Request, extract::State, middleware::Next};
use tokio::time::Instant;

pub async fn handle_audit_log_middleware(
    State(state): State<AppState>,
    Extension(current_user): Extension<Option<CurrentUser>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let module = path.split('/').nth(1).unwrap_or("unknown").to_string();
    let start = Instant::now();

    let response = next.run(request).await;

    let response_time = start.elapsed().as_millis() as i32;
    let status = response.status().as_u16() as i32;

    let user_id = if let Some(user) = current_user {
        user.uuid
    } else {
        "0".to_string()
    };

    // 使用状态中的数据库连接
    let audit_log = AuditLogService::new(state.clone());

    tokio::spawn(async move {
        if let Err(e) = audit_log
            .log(user_id, module, method, path, status, response_time)
            .await
        {
            tracing::error!("Failed to log audit: {}", e);
        }
    });

    Ok(response)
}
