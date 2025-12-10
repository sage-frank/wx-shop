// 确保在 main.rs 开头引入 lib
mod handler;
mod models;
mod repos;
mod router;
mod service;

use axum::Router;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// 配置文件路径
    #[arg(short, long, default_value = "Settings.toml")]
    conf: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(std::io::stdout());
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO) // 建议将级别调到 INFO 或 WARN，避免 DEBUG 级别日志过多
        .with_writer(non_blocking_appender) // 将日志输出导向非阻塞写入器
        .init();

    // --- 1. 加载配置 ---
    let settings = match wx_shop::Settings::new(&args.conf) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("加载配置文件失败: {}", e);
            return;
        }
    };

    // --- 2. 初始化数据库连接池 ---
    let pool = match settings.get_database_pool().await {
        Ok(p) => {
            // println!("✅ Database connection pool created successfully.");
            tracing::info!("Database connection pool created successfully.");
            p
        }
        Err(e) => {
            // eprintln!("❌ Failed to connect to database: {}", e);
            tracing::error!("Failed to connect to database: {}", e);
            return;
        }
    };

    // --- 3. 依赖实例化与注入 (依赖倒置的入口) ---
    // 创建 Repositories，并注入数据库连接池
    let user_repo = repos::users::UserRepository::new(pool.clone()); // 注意：使用 pool.clone()
    // 创建 Services，并注入 Repositories
    let user_service = service::users::UserService::new(user_repo);

    // --- 4. 路由合并与依赖挂载 ---
    let app = Router::new()
        .merge(router::user_routes())
        .with_state(user_service)
        // .layer(Extension(user_service))
        .layer(tower_http::trace::TraceLayer::new_for_http());

    // --- 5. 启动服务 ---
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
