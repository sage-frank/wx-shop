use wx_shop::{
    api,
    domain::services::{
        inventory::new_inventory_service,
        orders::new_order_service,
        products::new_product_service,
        users::new_user_service,
    },
    infra::repository,
    AppState, Settings,
};

use axum::{Router, body::Body};
use clap::Parser;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_sessions::{Expiry, Session, SessionManagerLayer, cookie::time::Duration};
use tower_sessions_redis_store::{RedisStore, fred::clients::Pool as RedisPool};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::writer::MakeWriterExt;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// 配置文件路径
    #[arg(short, long, default_value = "Settings.toml")]
    conf: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // 1. 加载配置
    let settings = Settings::new(&args.conf)?;

    // 2. 初始化日志 (保留 guards 防止日志线程过早销毁)
    let _guards = init_tracing(&settings);

    // 3. 初始化资源
    let pool = init_db_pool(&settings).await?;
    let redis_pool = init_redis_pool(&settings).await?;

    // 4. 依赖注入
    let user_repo = repository::users::UserRepository::new(pool.clone());
    let user_service = new_user_service(user_repo);

    let order_repo = repository::orders::OrderRepository::new(pool.clone());
    let order_service = new_order_service(order_repo);

    let product_repo = repository::products::ProductRepository::new(pool.clone());
    let s3_client = settings.get_s3_client();
    let product_service = new_product_service(product_repo, s3_client, settings.s3.bucket.clone());

    let inventory_repo = repository::inventory::InventoryRepository::new(pool.clone());
    let inventory_service = new_inventory_service(inventory_repo);

    let app_state = AppState {
        user_service,
        order_service,
        product_service,
        inventory_service,
    };

    // 5. 构建路由与中间件
    let session_store = RedisStore::new(redis_pool);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(3600)));

    let app = Router::new()
        .merge(api::routes::routes())
        .with_state(app_state)
        .layer(axum::middleware::from_fn(api::middleware::print_request_body))
        .layer(build_trace_layer())
        .layer(session_layer);

    // 6. 启动服务
    let addr = format!("{}:{}", settings.server.host, settings.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on http://{}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn init_tracing(settings: &Settings) -> Vec<WorkerGuard> {
    let file_appender = tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix(&settings.log.file)
        .max_log_files(settings.log.max_history)
        .build(&settings.log.dir)
        .expect("failed to initialize rolling file appender");

    let (non_blocking_file, guard_file) = tracing_appender::non_blocking(file_appender);
    let (non_blocking_stdout, guard_stdout) = tracing_appender::non_blocking(std::io::stdout());

    let log_level = settings
        .log
        .level
        .parse::<tracing::Level>()
        .unwrap_or(tracing::Level::INFO);

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_writer(non_blocking_stdout.and(non_blocking_file))
        .init();

    vec![guard_file, guard_stdout]
}

async fn init_db_pool(
    settings: &Settings,
) -> Result<sqlx::MySqlPool, Box<dyn std::error::Error>> {
    let pool = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        settings.get_database_pool(),
    )
    .await??;

    tracing::info!("Database connection pool created successfully.");
    Ok(pool)
}

async fn init_redis_pool(
    settings: &Settings,
) -> Result<RedisPool, Box<dyn std::error::Error>> {
    let pool = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        settings.get_redis_pool(),
    )
    .await
    .map_err(|_| "Redis connection timeout")??;

    tracing::info!("Redis connection pool created successfully.");
    Ok(pool)
}

fn trace_make_span(request: &axum::http::Request<Body>) -> tracing::Span {
    let session_id = request
        .extensions()
        .get::<Session>()
        .and_then(|s: &Session| s.id().map(|id| id.to_string()))
        .unwrap_or_else(|| "N/A".to_string());

    tracing::info_span!(
        "request",
        method = %request.method(),
        uri = %request.uri(),
        session_id = %session_id,
    )
}

fn trace_on_request(_request: &axum::http::Request<Body>, _span: &tracing::Span) {
    tracing::info!("started processing request");
}

fn trace_on_response(
    _response: &axum::http::Response<Body>,
    latency: std::time::Duration,
    _span: &tracing::Span,
) {
    tracing::info!("finished processing request in {:?}", latency);
}

type ShopTraceLayer = TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    fn(&axum::http::Request<Body>) -> tracing::Span,
    fn(&axum::http::Request<Body>, &tracing::Span),
    fn(&axum::http::Response<Body>, std::time::Duration, &tracing::Span),
>;

fn build_trace_layer() -> ShopTraceLayer {
    TraceLayer::new_for_http()
        .make_span_with(trace_make_span as fn(&axum::http::Request<Body>) -> tracing::Span)
        .on_request(trace_on_request as fn(&axum::http::Request<Body>, &tracing::Span))
        .on_response(
            trace_on_response
                as fn(&axum::http::Response<Body>, std::time::Duration, &tracing::Span),
        )
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("termination signal received");
}
