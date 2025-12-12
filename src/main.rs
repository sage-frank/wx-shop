// 确保在 main.rs 开头引入 lib
mod handler;
mod models;
mod repos;
mod router;
mod service;

use axum::body::Bytes;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum::{body::Body, Router};
use clap::Parser;
use http_body_util::BodyExt;
use tower_http::trace::TraceLayer;

// use tower_http::trace::TraceLayer;
use tower_sessions::cookie::time::Duration;

use tower_sessions::{Expiry, Session, SessionManagerLayer};
use tower_sessions_redis_store::RedisStore;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use axum::extract::FromRef;
use crate::service::users::UserService; // 确保路径正确


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// 配置文件路径
    #[arg(short, long, default_value = "Settings.toml")]
    conf: String,
}

#[derive(Clone)]
pub struct AppState {
    pub user_service: UserService,
    //
}

impl FromRef<AppState> for UserService {
    fn from_ref(state: &AppState) -> Self {
        state.user_service.clone()
    }
}


#[tokio::main]
async fn main() {
    let args = Args::parse();

    // --- 1. 加载配置 ---
    // 为了尽早加载日志配置，我们需要先加载 Settings
    // 但如果加载失败，我们暂时只能输出到标准输出，或者 panic
    let settings = match wx_shop::Settings::new(&args.conf) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("加载配置文件失败: {}", e);
            return;
        }
    };

    // 配置日志文件轮转
    let file_appender = tracing_appender::rolling::daily(&settings.log.dir, &settings.log.file);
    let (non_blocking_file, _guard_file) = tracing_appender::non_blocking(file_appender);
    let (non_blocking_stdout, _guard_stdout) = tracing_appender::non_blocking(std::io::stdout());

    let log_level = settings.log.level.parse::<tracing::Level>().unwrap_or(tracing::Level::INFO);

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_writer(non_blocking_stdout.and(non_blocking_file))
        .init();

    // --- 2. 初始化数据库连接池 ---
    let pool = match settings.get_database_pool().await {
        Ok(p) => {
            tracing::info!("Database connection pool created successfully.");
            p
        }
        Err(e) => {
            tracing::error!("Failed to connect to database: {}", e);
            return;
        }
    };

    // --- 3. 依赖实例化与注入 (依赖倒置的入口) ---
    // Redis Session
    let redis_pool = match settings.get_redis_pool().await {
        Ok(pool) => {
            tracing::info!("Redis connection pool created successfully.");
            pool
        }
        Err(e) => {
            tracing::error!("Failed to connect to redis: {}", e);
            return;
        }
    };

    let session_store = RedisStore::new(redis_pool);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(3600)));

    // 创建 Repositories，并注入数据库连接池
    let user_repo = repos::users::UserRepository::new(pool.clone()); // 注意：使用 pool.clone()
    // 创建 Services，并注入 Repositories
    let user_service = UserService::new(user_repo);

    let app_state = AppState {
        user_service
    };

    // --- 4. 路由合并与依赖挂载 ---
    let app = Router::new()
        .merge(router::user_routes())
        .with_state(app_state)
        .layer(axum::middleware::from_fn(print_request_body))
        .layer(session_layer)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<Body>| {
                    // 尝试从请求的 extensions 中获取 Session 实例
                    let session_id = request
                        .extensions()
                        .get::<Session>()
                        .and_then(|s| s.id().map(|id| id.to_string()))
                        .unwrap_or_else(|| "N/A".to_string());

                    tracing::info_span!(
                "request",
                method = %request.method(),
                uri = %request.uri(),
                session_id = %session_id,
                )}
                ));

        // .layer(
        //     TraceLayer::new_for_http()
        //         .make_span_with(|request: &axum::http::Request<axum::body::Body>| {
        //             let session_id = request
        //                 .extensions()
        //                 .get::<Session>()
        //                 .map(|s| s.id().map(|id| id.to_string()).unwrap_or_default())
        //                 .unwrap_or_default();
        //
        //             tracing::info_span!(
        //                 "request",
        //                 method = %request.method(),
        //                 uri = %request.uri(),
        //                 session_id = %session_id
        //             )
        //         })
        //         .on_request(
        //             |_request: &axum::http::Request<axum::body::Body>, _span: &tracing::Span| {
        //                 tracing::info!("started processing request");
        //             },
        //         )
        //         .on_response(
        //             |_response: &axum::http::Response<axum::body::Body>,
        //              latency: std::time::Duration,
        //              _span: &tracing::Span| {
        //                 tracing::info!("finished processing request in {:?}", latency);
        //             },
        //         ),
        // );

    // --- 5. 启动服务 ---
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn print_request_body(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
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
