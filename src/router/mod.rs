use std::sync::Arc;
use axum::{routing::get, Router};
use axum::extract::FromRef;
pub(crate) use crate::handler::{index, users};
use crate::service::users::UserService;
// 导入 Handler

// 封装 User 相关的路由
pub fn user_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
// S 必须能够从中提取出 Arc<UserService>
    Arc<UserService>: FromRef<S>,
{
    Router::new()
        .route("/users/{id}", get(users::get_user_handler))
        .route("/",get(index::index))
}