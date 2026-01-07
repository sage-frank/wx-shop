use axum::routing::{get, post, put, delete};
use axum::Router;
use crate::AppState;
use crate::handler::users;
use crate::handler::orders;
use crate::router::middleware;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/user/{id}", get(users::get_user_by_id_handler))
        .route("/orders", post(orders::create_order_handler))
        .route("/orders/{order_id}", delete(orders::delete_order_handler))
        .route("/orders/{order_id}", put(orders::update_order_handler))
        .route("/orders/{order_id}", get(orders::get_order_handler))
        .route("/orders/{order_id}/items", get(orders::get_order_items_handler))
        .route_layer(axum::middleware::from_fn(middleware::require_login))
}
