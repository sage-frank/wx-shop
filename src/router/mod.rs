use axum::{routing::get, Router};
pub(crate) use crate::handler::{index, users}; // 导入 Handler

// 封装 User 相关的路由
pub fn user_routes() -> Router {
    Router::new()
        .route("/users/{id}", get(users::get_user_handler))
        .route("/",get(index::index))
}