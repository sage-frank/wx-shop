// 确保在 main.rs 开头引入 lib
mod handler;
mod repos;
mod router;
mod service;
mod models;

use axum::Extension;
use axum::Router;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // --- 1. 加载配置 ---
    let settings = match wx_shop::Settings::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            return;
        }
    };

    // --- 2. 初始化数据库连接池 ---
    let pool = match settings.get_database_pool().await {
        Ok(p) => {
            println!("✅ Database connection pool created successfully.");
            p
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to database: {}", e);
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
        // .route(
        //     "/",
        //     axum::routing::get(|| async { "Refactored API is running!" }),
        // )
        .layer(Extension(user_service))
        // ... 添加 TraceLayer ...
        .layer(tower_http::trace::TraceLayer::new_for_http());

    // --- 5. 启动服务 ---
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
