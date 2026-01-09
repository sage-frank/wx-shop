use crate::models;
use axum::body::{Body, Bytes};
use axum::{Json, http::StatusCode, response::IntoResponse};
use axum::{extract::Request, middleware::Next};
use http_body_util::BodyExt;
use serde_json::json;
use tower_sessions::Session;

pub async fn require_login(
    request: Request,
    next: Next,
) -> Result<axum::response::Response, StatusCode> {
    let session = request.extensions().get::<Session>();

    // 尝试获取用户，如果中间任何一步失败（没有 session，或是 session 里没用户），则返回 None
    let mut user_exists = false;

    if let Some(s) = session
        && let Ok(Some(_)) = s.get::<models::User>("user").await
    {
        user_exists = true;
    }

    if !user_exists {
        return Ok(Json(json!({"code": 4010, "msg": "not logged in", "data":{}})).into_response());
    }

    Ok(next.run(request).await)
}

pub async fn print_request_body(
    request: Request,
    next: Next,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    if !tracing::enabled!(tracing::Level::DEBUG) {
        return Ok(next.run(request).await);
    }

    let (parts, body) = request.into_parts();
    let bytes = buffer_and_print("request", body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));
    let res = next.run(req).await;
    Ok(res)
}

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, axum::http::StatusCode>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    const MAX_BODY_LOG_BYTES: usize = 2048;

    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            tracing::error!("Failed quest:{err}");
            return Err(axum::http::StatusCode::BAD_REQUEST);
        }
    };

    let preview = if bytes.len() > MAX_BODY_LOG_BYTES {
        &bytes[..MAX_BODY_LOG_BYTES]
    } else {
        &bytes
    };

    if let Ok(body_str) = std::str::from_utf8(preview) {
        tracing::debug!("{} body = {:?}", direction, body_str);
    }

    Ok(bytes)
}
