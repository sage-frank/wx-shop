use crate::AppState;
use crate::handler::{index, users};
use axum::Router;
use axum::routing::{get, post};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(users::login_handler))
        .route("/debug/hash", post(users::hash_handler))
        .route("/", get(index::index))
}
