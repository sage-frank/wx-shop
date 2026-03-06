use crate::AppState;
use crate::domain::services::ServiceError;
use axum::extract::Path;
use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json;
use sha2::{Digest, Sha256};
use tracing::{instrument, info, warn};

#[derive(Deserialize)]
pub struct HashReq {
    pub passwd: String,
    pub salt: String,
}

#[instrument(level = "debug", skip(payload))]
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

#[instrument(level = "debug", skip(session, app_state, payload), fields(username = payload.username.as_str()))]
pub async fn login_handler(
    session: tower_sessions::Session,
    State(app_state): State<AppState>,
    Json(payload): Json<LoginReq>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!(username = payload.username.as_str(), "login start");
    match app_state
        .user_service
        .login(&payload.username, &payload.passwd)
        .await
    {
        Ok(user) => {
            if let Err(e) = session.insert("user", user).await {
                Ok(Json(serde_json::json!({
                    "code": 5000,
                    "msg": format!("Session error: {}", e)
                })))
            } else {
                info!(username = payload.username.as_str(), "login success");
                Ok(Json(serde_json::json!({
                    "code": 0,
                    "msg": "login success"
                })))
            }
        }

        Err(e) => {
            warn!(username = payload.username.as_str(), error = %e, "login failed");
            Ok(Json(serde_json::json!({
                "code": 4001,
                "msg": e
            })))
        },
    }
}

#[instrument(level = "debug", skip(app_state), fields(id))]
pub async fn get_user_by_id_handler(
    State(app_state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let user = app_state.user_service.find_user_by_id(id).await?;

    Ok(Json(serde_json::json!({
        "code": 0,
        "msg": "success",
        "data": user
    })))
}

#[instrument(level = "debug", skip(session))]
pub async fn logout_handler(session: tower_sessions::Session) -> Result<Json<serde_json::Value>, StatusCode> {
    session.delete().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({
        "code": 0,
        "msg": "logout success"
    })))
}
