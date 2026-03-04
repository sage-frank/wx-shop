use crate::AppState;
use crate::api::handlers::{orders, users, products, inventory};
use crate::api::middleware::require_login;

use axum::Router;
use axum::routing::{get, post, put};

pub fn routes() -> Router<AppState> {
    let protected_routes = Router::new()
        .route("/logout", post(users::logout_handler))
        .route("/user/{id}", get(users::get_user_by_id_handler))
        .route("/orders", post(orders::create_order_handler))
        .route(
            "/orders/{order_id}",
            get(orders::get_order_handler)
                .put(orders::update_order_handler)
                .delete(orders::delete_order_handler),
        )
        .route("/orders/{order_id}/cancel", post(orders::cancel_order_handler))
        .route(
            "/orders/{order_id}/items",
            get(orders::get_order_items_handler),
        )
        .route(
            "/products",
            get(products::list_products_handler).post(products::create_product_handler),
        )
        .route("/products/{product_id}", put(products::update_product_handler))
        .route(
            "/products/{product_id}/off-shelf",
            post(products::off_shelf_product_handler),
        )
        .route("/products/upload", post(products::upload_image_handler))
        .route("/inventory", get(inventory::list_inventory_handler))
        .route(
            "/inventory/{inv_id}",
            put(inventory::update_inventory_handler).get(inventory::get_inventory_handler),
        )
        .route_layer(axum::middleware::from_fn(require_login));

    // Public routes
    Router::new()
        .route("/login", post(users::login_handler))
        .merge(protected_routes)
}
