use axum::routing::get;
use axum::Router;
use crate::AppState;
use crate::handler::users;
use crate::router::middleware;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/user/{id}", get(users::get_user_by_id_handler))
        .route_layer(axum::middleware::from_fn(middleware::require_login))
}

