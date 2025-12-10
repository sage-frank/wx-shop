use axum::{extract::Path, http::StatusCode, Json};

use crate::service::users::UserService;
use serde::Serialize;
use std::sync::Arc;
use axum::extract::State;

#[derive(Serialize)]
pub struct UserResp {
    id: u64,
    name: String,
    email: String,
}

// 定义响应结构体
#[derive(Serialize)]
pub struct UserResponse {
    msg: String,
    code: u32,
    data: Vec<UserResp>,
}

// Handler 函数，通过 Extension 获取 UserService
pub async fn get_user_handler(
    // Path 提取器
    Path(user_id): Path<u64>,
    // 注入 UserService 依赖：使用 State 提取 Arc<UserService>
    State(user_service): State<Arc<UserService>>, // <-- 正确！
) -> Result<Json<UserResponse>, StatusCode> {
    // 调用 Service 层处理业务逻辑
    match user_service.get_user(user_id).await {
        Some(user) => {
            println!("-> Handler: Successfully found user {}", user_id);
            let resp_user = UserResp {
                id: user_id,
                name: user.name.clone(),
                email: user.email.unwrap(),
            };

            Ok(Json(UserResponse {
                code: 0,
                msg: "success".to_string(),
                data: vec![resp_user],
            }))
        }
        None => Ok(Json(UserResponse {
            code: 4000,
            msg: "not found".to_string(),
            data: vec![],
        })),
    }
}
