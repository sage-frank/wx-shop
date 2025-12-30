use axum::{http::StatusCode, response::IntoResponse, Json};

pub async fn handler_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "code": 404,
            "msg": "not found"
        })),
    )
}

