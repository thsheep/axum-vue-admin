use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use std::fmt;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T, status_code: StatusCode) -> Self {
        Self {
            code: status_code.as_u16(),
            message: "success".to_string(),
            data: Some(data),
        }
    }

    pub fn success_empty(status_code: StatusCode) -> Self {
        Self {
            code: status_code.as_u16(),
            message: "success".to_string(),
            data: None,
        }
    }

    pub fn error(code: u16, msg: String) -> Self {
        Self {
            code,
            message: msg,
            data: None,
        }
    }
}

// 为 ApiResponse 实现 IntoResponse
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::from_u16(self.code).unwrap(), Json(self)).into_response()
    }
}

impl<T: Serialize> fmt::Debug for ApiResponse<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiResponse")
            .field("code", &self.code)
            .field("msg", &self.message)
            .finish()
    }
}
