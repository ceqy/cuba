use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod clients;
mod dto;
mod handlers;
mod state;

use clients::auth::AuthClient;
use state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "api_gateway=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2. 加载配置
    // dotenvy::dotenv().ok();
    let auth_service_url = std::env::var("AUTH_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());
    let port = std::env::var("GATEWAY_PORT").unwrap_or_else(|_| "8050".to_string()).parse::<u16>()?;

    tracing::info!("Connecting to Auth Service at {}...", auth_service_url);

    // 3. 初始化 gRPC 客户端
    let auth_client = AuthClient::connect(auth_service_url).await?;

    let state = AppState {
        auth_client,
    };

    // 4. 构建路由
    let app = Router::new()
        // Health Check
        .route("/health", get(handlers::health::health_check))
        // Auth Routes
        .route("/api/v1/auth/register", post(handlers::auth::register))
        .route("/api/v1/auth/login", post(handlers::auth::login))
        .route("/api/v1/auth/refresh", post(handlers::auth::refresh))
        .route("/api/v1/auth/me", get(handlers::auth::me))
        // Shared State
        .with_state(state)
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any) // 生产环境请限制 Origin
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // 5. 启动服务
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("API Gateway listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
