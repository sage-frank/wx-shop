use axum::routing::{get, post};
use axum::Router;
use crate::AppState;
use crate::handler::{index, users};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(users::login_handler))
        .route("/debug/hash", post(users::hash_handler))
        .route("/", get(index::index))
}

