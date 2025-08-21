use crate::services::sse::SSEService;

use crate::errors::app_error::AppError;
use crate::schemas::auth::CurrentUser;
use axum::extract::State;
use axum::{response::sse::{Event, Sse}, Extension};
use futures_util::Stream;

pub async fn global_message_push(
    State(service): State<SSEService>,
    Extension(current_user): Extension<CurrentUser>,
) -> Sse<impl Stream<Item = Result<Event, AppError>>> {
    service.global_message(current_user).await
}