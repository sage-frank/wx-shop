use crate::domain::models;
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
    let (parts, body) = request.into_parts();

    // 记录 Path 和 Query
    let path = parts.uri.path().to_string();
    let query = parts.uri.query().unwrap_or("").to_string();
    let method = parts.method.clone();

    // 提取 Content-Type 用于后续判断
    let content_type = parts
        .headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    tracing::info!(
        "Processing request: method={} path={} query={}",
        method,
        path,
        query
    );

    // 如果不是 debug 模式，或者文件上传，跳过 Body 读取
    if !tracing::enabled!(tracing::Level::DEBUG) || content_type.starts_with("multipart/form-data") {
        if content_type.starts_with("multipart/form-data") {
            tracing::debug!("Body: [Multipart data, skipping log]");
        } else {
            tracing::debug!("Body: [Skipping log (not debug level)]");
        }
        let req = Request::from_parts(parts, body);
        return Ok(next.run(req).await);
    }

    // 读取 Body (设置上限，例如 1MB，防止内存耗尽)
    const MAX_BODY_SIZE: usize = 1024 * 1024; // 1MB

    // 由于 axum body 是 stream，我们需要收集它
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            tracing::error!("Failed to read request body: {err}");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // 异步记录日志
    if bytes.len() > MAX_BODY_SIZE {
        let len = bytes.len();
        // 直接打印，避免 spawn
        tracing::debug!("Body: [Too large to log, size={}]", len);
    } else {
        // 直接打印
        log_body_content(&bytes, &content_type).await;
    }

    // 重构 Request 继续处理
    let req = Request::from_parts(parts, Body::from(bytes));
    Ok(next.run(req).await)
}

async fn log_body_content(bytes: &Bytes, content_type: &str) {
    // 尝试解析并打印
    // 优先尝试 JSON
    if content_type.contains("json") {
        if let Ok(mut json_val) = serde_json::from_slice::<serde_json::Value>(bytes) {
            // 截断大字段
            truncate_json_strings(&mut json_val, 200); // 字符串超过 200 字符就截断
            tracing::debug!("Body (JSON): {}", json_val);
        } else {
            // JSON 解析失败，回退到字符串打印
            log_raw_string(bytes);
        }
    } else if content_type.contains("text")
        || content_type.contains("xml")
        || content_type.contains("x-www-form-urlencoded")
    {
        log_raw_string(bytes);
    } else {
        // 二进制或其他
        tracing::debug!(
            "Body (Other): [Content type: {}, size={}]",
            content_type,
            bytes.len()
        );
    }
}

fn log_raw_string(bytes: &[u8]) {
    const MAX_LOG_CHARS: usize = 1000;
    let s = String::from_utf8_lossy(bytes);
    if s.len() > MAX_LOG_CHARS {
        tracing::debug!("Body (Raw): {}... [TRUNCATED]", &s[..MAX_LOG_CHARS]);
    } else {
        tracing::debug!("Body (Raw): {}", s);
    }
}

fn truncate_json_strings(v: &mut serde_json::Value, max_len: usize) {
    match v {
        serde_json::Value::String(s) => {
            if s.len() > max_len {
                *s = format!("{}...[TRUNCATED, len={}]", &s[..max_len], s.len());
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                truncate_json_strings(item, max_len);
            }
        }
        serde_json::Value::Object(map) => {
            for (_, val) in map {
                truncate_json_strings(val, max_len);
            }
        }
        _ => {}
    }
}
