// 确保在 main.rs 开头引入 lib
mod domain;
mod handler;
mod models;
mod repos;
mod router;
mod service;

use axum::{Router, body::Body};
use clap::Parser;
use tower_http::trace::TraceLayer;

use tower_sessions::cookie::time::Duration;

use crate::service::orders::{OrderService, new_order_service};
use crate::service::users::{UserService, new_user_service};
use axum::extract::FromRef;
use std::sync::Arc;
use tower_sessions::{Expiry, Session, SessionManagerLayer};
use tower_sessions_redis_store::RedisStore;
use tracing_subscriber::fmt::writer::MakeWriterExt;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// 配置文件路径
    #[arg(short, long, default_value = "Settings.toml")]
    conf: String,
}

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<dyn UserService>,
    pub order_service: Arc<dyn OrderService>,
}

impl FromRef<AppState> for Arc<dyn UserService> {
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

    let log_level = settings
        .log
        .level
        .parse::<tracing::Level>()
        .unwrap_or(tracing::Level::INFO);

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

    // user repository, user service
    let user_repo = repos::users::UserRepository::new(pool.clone()); // 注意：使用 pool.clone()
    let user_service = new_user_service(user_repo);

    // order repository, user service
    let order_repo = repos::orders::OrderRepository::new(pool.clone());
    let order_service = new_order_service(order_repo);

    let app_state = AppState {
        user_service,
        order_service,
    };

    // --- 4. 路由合并与依赖挂载 ---
    let app = Router::new()
        .merge(router::routes())
        .with_state(app_state)
        .layer(axum::middleware::from_fn(
            router::middleware::print_request_body,
        ))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<Body>| {
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
                    )
                })
                .on_request(
                    |_request: &axum::http::Request<Body>, _span: &tracing::Span| {
                        tracing::info!("started processing request");
                    },
                )
                .on_response(
                    |_response: &axum::http::Response<Body>,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        tracing::info!("finished processing request in {:?}", latency);
                    },
                ),
        )
        .layer(session_layer);

    // --- 5. 启动服务 ---
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
