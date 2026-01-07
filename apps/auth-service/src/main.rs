//! Auth Service - Main Entry Point
//!
//! CUBA Enterprise 认证服务

mod application;
mod domain;
mod grpc;
mod infrastructure;

// 引入生成的 proto 代码
pub mod proto {
    tonic::include_proto!("auth");
}

use grpc::AuthServiceImpl;
use infrastructure::persistence::{PgRoleRepository, PgUserRepository, PgRefreshTokenRepository, PgVerificationRepository, PgApiKeyRepository, PgAuditLogRepository, PgSessionRepository, PgClientRepository, PgPolicyRepository};
use infrastructure::services::{JwtService, social_auth::SocialAuthService, sso_service::SSOService};
use proto::auth_service_server::AuthServiceServer;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing::info;

/// 服务配置
#[derive(Debug, serde::Deserialize)]
struct AuthServiceConfig {
    server_addr: String,
    database_url: String,
    jwt_secret: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    cuba_telemetry::init_subscriber("auth-service");

    // 加载配置
    let mut config: AuthServiceConfig = cuba_config::load_config("auth-service")?;
    
    // 优先使用环境变量
    if let Ok(addr) = std::env::var("SERVER_ADDR") {
        config.server_addr = addr;
    }
    
    info!("Starting auth-service on {}", config.server_addr);

    // 创建数据库连接池
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;
    let pool = Arc::new(pool);

    // 创建仓储
    let user_repo = Arc::new(PgUserRepository::new(pool.clone()));
    let role_repo = Arc::new(PgRoleRepository::new(pool.clone()));
    let refresh_token_repo = Arc::new(PgRefreshTokenRepository::new(pool.clone()));
    let verification_repo = Arc::new(PgVerificationRepository::new(pool.clone()));
    let api_key_repo = Arc::new(PgApiKeyRepository::new(pool.clone()));
    let audit_repo = Arc::new(PgAuditLogRepository::new(pool.clone()));
    let session_repo = Arc::new(PgSessionRepository::new(pool.clone()));
    let session_repo = Arc::new(PgSessionRepository::new(pool.clone()));
    let client_repo = Arc::new(PgClientRepository::new(pool.clone()));
    let policy_repo = Arc::new(PgPolicyRepository::new(pool.clone()));

    // 创建 Token 服务
    let token_service = Arc::new(JwtService::new(
        Default::default(), // 或者从环境变量读取配置
        user_repo.clone(),
        refresh_token_repo,
    ));

    // 创建 Social Auth Service
    let social_auth_service = Arc::new(SocialAuthService::new(
        std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default(),
        std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default(),
    ));

    let sso_service = Arc::new(SSOService::new());

    // 创建 gRPC 服务
    let auth_service = AuthServiceImpl::new(
        user_repo, 
        role_repo, 
        verification_repo, 
        api_key_repo, 
        audit_repo, 
        session_repo, 
        client_repo,
        policy_repo,
        token_service,
        social_auth_service,
        sso_service,
    );

    // 解析地址
    let addr: std::net::SocketAddr = config.server_addr.parse()?;

    info!("Auth service listening on {}", addr);

    // 启动 gRPC 服务器
    tonic::transport::Server::builder()
        .add_service(AuthServiceServer::new(auth_service))
        .serve(addr)
        .await?;

    Ok(())
}
