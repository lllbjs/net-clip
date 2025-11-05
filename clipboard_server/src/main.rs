mod config;
mod database;
mod handlers;

use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::Config;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载配置
    let config = Config::from_env();
    tracing::info!("配置加载成功 ✅");

    // 初始化数据库连接池
    let pool = database::init_pool(&config)
        .await
        .expect("无法创建 MySQL 连接池，请检查：1. 数据库地址/密码正确 2. MySQL 服务已启动 3. 数据库已创建");
    tracing::info!("MySQL 连接池初始化成功 ✅");

    // 配置路由
    let app = Router::new()
        .route("/", get(handlers::root))
        .route("/health", get(handlers::health_check))
        .with_state(pool)
        .layer(tower_http::cors::CorsLayer::permissive());

    // 启动服务
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("服务端启动成功，地址：http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("无法绑定端口，请检查端口是否被占用");

    axum::serve(listener, app)
        .await
        .expect("服务端启动失败");
}