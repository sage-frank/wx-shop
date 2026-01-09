pub mod orders;
pub mod users;

use axum::response::{IntoResponse, Response};
use axum::{Json, http::StatusCode};
use serde_json::json;
#[derive(Debug)]
pub enum ServiceError {
    NotFound(String),      // 业务错误：找不到资源
    Database(sqlx::Error), // 基础设施错误：数据库操作失败
}

// 实现 From trait，让 ? 操作符可以自动转换
impl From<sqlx::Error> for ServiceError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => ServiceError::NotFound("resource not found".into()),
            other => ServiceError::Database(other),
        }
    }
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        match self {
            ServiceError::NotFound(msg) => {
                // 业务错误：404 Not Found
                (
                    StatusCode::NOT_FOUND,
                    Json(json!({"code": 4040, "msg": msg, "data":{}})),
                )
                    .into_response()
            }
            _ => {
                // 内部错误：500 Internal Server Error
                tracing::error!("Unhandled service error: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
        }
    }
}
