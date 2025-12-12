use axum::{routing::{get, post}, Router, http::StatusCode, response::IntoResponse, Json};
use axum::{middleware::Next, extract::Request};
use tower_sessions::Session;
use serde_json::json;
use crate::models;
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
    
    let protected = Router::new()
        .route("/user/{id}", get(users::get_user_by_id_handler))
        .route_layer(axum::middleware::from_fn(require_login));

    Router::new()
        .route("/login", post(users::login_handler))
        .merge(protected)
        .route("/debug/hash", post(users::hash_handler))
        .route("/", get(index::index))
        .fallback(handler_404)
}


async fn require_login(
    request: Request,
    next: Next,
) -> Result<axum::response::Response, StatusCode> {
    let logged_in = if let Some(session) = request.extensions().get::<Session>() {
        match session.get::<models::User>("user").await {
            Ok(Some(_)) => true,
            _ => false,
        }
    } else {
        false
    };

    if !logged_in {
        return Ok(Json(json!({"code": 4010, "msg": "not logged in"})).into_response());
    }

    Ok(next.run(request).await)
}
