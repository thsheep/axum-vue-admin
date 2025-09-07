//  分页响应的Response

use axum::{Json, http::StatusCode, response::IntoResponse, response::Response};
use serde::Serialize;
use std::fmt;

#[derive(Serialize)]
pub struct PaginatedApiResponse<T: Serialize> {
    pub code: u16,
    pub msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    pub total: u64,
    pub page: u64,
    #[serde(rename = "pageSize")]
    pub page_size: u64,
}

impl<T: Serialize> PaginatedApiResponse<T> {
    pub fn success(
        data: T,
        total: u64,
        page: u64,
        page_size: u64,
        status_code: StatusCode,
    ) -> Self {
        Self {
            code: status_code.as_u16(),
            msg: "success".to_string(),
            data: Some(data),
            total,
            page,
            page_size,
        }
    }
    
}

// 为 PaginatedApiResponse 实现 IntoResponse
impl<T: Serialize> IntoResponse for PaginatedApiResponse<T> {
    fn into_response(self) -> Response {
        (StatusCode::from_u16(self.code).unwrap(), Json(self)).into_response()
    }
}

impl<T: Serialize> fmt::Debug for PaginatedApiResponse<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PaginatedApiResponse")
            .field("code", &self.code)
            .field("msg", &self.msg)
            .field("total", &self.total)
            .field("page", &self.page)
            .field("page_size", &self.page_size)
            .finish()
    }
}
