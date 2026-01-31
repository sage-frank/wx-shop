pub mod orders;
pub mod users;
pub mod products;
pub mod inventory;

use axum::response::{IntoResponse, Response};
use axum::{Json, http::StatusCode};
use serde_json::json;

#[derive(Debug)]
pub enum ServiceError {
    NotFound(String),
    Database(sqlx::Error),
    Internal(String),
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ServiceError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ServiceError::Database(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            ServiceError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        let body = Json(json!({
            "code": -1,
            "msg": error_message,
        }));
        (status, body).into_response()
    }
}

impl From<sqlx::Error> for ServiceError {
    fn from(err: sqlx::Error) -> Self {
        ServiceError::Database(err)
    }
}
