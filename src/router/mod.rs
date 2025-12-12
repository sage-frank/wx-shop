use axum::{routing::{get, post}, Router, http::StatusCode, response::IntoResponse, Json};
use crate::handler::{index, users};
use crate::AppState;

// 404 handler
async fn handler_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "code": 404,
            "msg": "not found"
        })),
    )
}

pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(users::login_handler))
        .route("/user/{id}", get(users::get_user_by_id_handler))
        .route("/debug/hash", post(users::hash_handler))
        .route("/", get(index::index))
        .fallback(handler_404)
}
