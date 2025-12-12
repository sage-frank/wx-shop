// use crate::service::users::UserService;
use axum::{extract::State, http::StatusCode, Json};
use axum::extract::Path;
use serde::Deserialize;
use serde_json;
use sha2::{Sha256, Digest};
use crate::AppState;
use crate::service::ServiceError;

#[derive(Deserialize)]
pub struct HashReq {
    pub passwd: String,
    pub salt: String,
}

pub async fn hash_handler(Json(payload): Json<HashReq>) -> Json<serde_json::Value> {
    let mut hasher = Sha256::new();
    hasher.update(payload.passwd.as_bytes());
    hasher.update(payload.salt.as_bytes());
    let hash = hex::encode(hasher.finalize());
    
    Json(serde_json::json!({
        "hash": hash
    }))
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub username: String,
    pub passwd: String,
}


pub async fn login_handler(
    session: tower_sessions::Session,
    State(app_state): State<AppState>,
    Json(payload): Json<LoginReq>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match app_state.user_service.login(&payload.username, &payload.passwd).await {
        Ok(user) => {
            if let Err(e) = session.insert("user", user).await {
                return Ok(Json(serde_json::json!({
                    "code": 5000,
                    "msg": format!("Session error: {}", e)
                })));
            } else {
                Ok(Json(serde_json::json!({
                    "code": 0,
                    "msg": "login success"
                })))
            }
        }

        Err(e) => Ok(Json(serde_json::json!({
            "code": 4001,
            "msg": e
        }))),
    }
}


pub async fn get_user_by_id_handler(
    State(app_state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let user = app_state.user_service.find_user_by_id(id).await?;
    Ok(Json(serde_json::json!({
        "code": 0,
        "data": user
    })))
}
