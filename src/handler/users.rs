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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::service::users::UserService;
    use crate::models;
    use std::future::Future;
    use std::pin::Pin;

    struct MockService;

    impl UserService for MockService {
        fn login<'a>(&'a self, _username: &'a str, _password: &'a str) -> Pin<Box<dyn Future<Output = Result<models::User, String>> + Send + 'a>> {
            Box::pin(async { Err("not implemented".into()) })
        }

        fn find_user_by_id<'a>(&'a self, id:u32) -> Pin<Box<dyn Future<Output = Result<models::User, ServiceError>> + Send + 'a>> {
            Box::pin(async move {
                Ok(models::User { id, username: "u".into(), passwd: "p".into(), salt: "s".into(), created_at: None, updated_at: None })
            })
        }
    }

    #[tokio::test]
    async fn test_get_user_by_id_handler() {
        let app_state = AppState { user_service: Arc::new(MockService) };
        let resp = get_user_by_id_handler(State(app_state), Path(1)).await.unwrap();
        let v = resp.0;
        assert_eq!(v["code"], 0);
        assert_eq!(v["data"]["id"], 1);
    }

    #[tokio::test]
    async fn test_hash_handler() {
        let payload = HashReq { passwd: "a".into(), salt: "b".into() };
        let resp = hash_handler(Json(payload)).await;
        let v = resp.0;
        assert!(v["hash"].is_string());
    }
}
