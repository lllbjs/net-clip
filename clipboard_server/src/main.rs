mod config;
mod database;
mod handlers;
mod middlewares;

use axum::{
    routing::get,
    Router,
    middleware,
    extract::Request,
    http::{StatusCode, HeaderMap},
};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::Config;
use database::DbPool;

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

    // 创建共享状态
    let shared_state = (pool.clone(), config.clone());

    // 需要认证的路由
    let auth_routes = Router::new()
        .route("/api/auth/me", get(handlers::auth::get_me))
        .route("/api/auth/logout", axum::routing::post(handlers::auth::logout))
        .route("/api/clips", axum::routing::post(handlers::clips::create_clip))
        .route("/api/clips", get(handlers::clips::get_user_clips))
        .route("/api/clips/:id", get(handlers::clips::get_clip_by_id))
        .route("/api/clips/:id", axum::routing::put(handlers::clips::update_clip))
        .route("/api/clips/:id", axum::routing::delete(handlers::clips::delete_clip))
        .layer(middleware::from_fn_with_state(
            shared_state.clone(),
            |state: axum::extract::State<(DbPool, Config)>, request: Request, next: middleware::Next| async move {
                let headers = request.headers().clone();
                middlewares::auth(state, headers, request, next).await
            }
        ));

    // 公开路由
    let public_routes = Router::new()
        .route("/", get(handlers::root))
        .route("/health", get(handlers::health_check))
        .route("/api/clips/:short_url", get(handlers::clips::get_clip_by_short_url))
        .route("/api/auth/register", axum::routing::post(handlers::auth::register))
        .route("/api/auth/login", axum::routing::post(handlers::auth::login))
        .route("/api/auth/refresh", axum::routing::post(handlers::auth::refresh_token));

    let app = Router::new()
        .merge(public_routes)
        .merge(auth_routes)
        .layer(tower_http::cors::CorsLayer::permissive());

    // 启动服务
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("服务端启动成功，地址：http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("无法绑定端口，请检查端口是否被占用");

    axum::serve(listener, app)

        .expect("服务端启动失败");
}