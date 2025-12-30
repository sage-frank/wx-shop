use axum::{http::StatusCode, response::IntoResponse, Json};
use axum::{middleware::Next, extract::Request};
use serde_json::json;
use tower_sessions::Session;
use crate::models;
use axum::body::{Body, Bytes};
use http_body_util::BodyExt;

pub async fn require_login(
    request: Request,
    next: Next,
) -> Result<axum::response::Response, StatusCode> {
    let logged_in = if let Some(session) = request.extensions().get::<Session>() {
        match session.get::<models::User>("user").await {
            Ok(Some(_)) => true,
            _ => false,
        }
    } else {
        false
    };

    if !logged_in {
        return Ok(Json(json!({"code": 4010, "msg": "not logged in"})).into_response());
    }

    Ok(next.run(request).await)
}

pub async fn print_request_body(
    request: Request,
    next: Next,
) -> Result<axum::response::Response, axum::http::StatusCode> {
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
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            tracing::error!("Failed quest:{err}");
            return Err(axum::http::StatusCode::BAD_REQUEST);
        }
    };

    if let Ok(body_str) = std::str::from_utf8(&bytes) {
        tracing::info!("{} body = {:?}", direction, body_str);
    }

    Ok(bytes)
}

