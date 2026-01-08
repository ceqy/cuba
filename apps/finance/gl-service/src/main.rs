//! Finance Service - Main Entry Point
//!
//! CUBA Enterprise 财务服务 - 总账凭证服务

mod application;
mod domain;
mod grpc;
mod infrastructure;
mod proto;

use grpc::GlJournalEntryServiceImpl;
use proto::finance::gl::gl_journal_entry_service_server::GlJournalEntryServiceServer;
use proto::finance::gl::FILE_DESCRIPTOR_SET;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing::info;

/// 服务配置
#[derive(Debug, serde::Deserialize)]
struct FinanceServiceConfig {
    server_addr: String,
    database_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    cuba_telemetry::init_subscriber("gl-service");

    // 加载配置
    let mut config: FinanceServiceConfig = cuba_config::load_config("gl-service")?;
    
    // 优先使用环境变量
    if let Ok(addr) = std::env::var("SERVER_ADDR") {
        config.server_addr = addr;
    }
    
    if let Ok(db_url) = std::env::var("DATABASE_URL") {
        config.database_url = db_url;
    }
    
    info!("Starting gl-service on {}", config.server_addr);

    // 创建数据库连接池
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;
    let pool = Arc::new(pool);

    // 创建 gRPC 服务
    let gl_service = GlJournalEntryServiceImpl::new(pool.clone());

    // --- gRPC 反射服务 ---
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()?;

    // --- gRPC 健康检查服务 ---
    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<GlJournalEntryServiceServer<GlJournalEntryServiceImpl>>()
        .await;

    // 解析地址
    let addr: std::net::SocketAddr = config.server_addr.parse()?;

    info!("Gl service listening on {}", addr);

    // 启动 gRPC 服务器 (带优雅停机)
    tonic::transport::Server::builder()
        .add_service(health_service)
        .add_service(reflection_service)
        .add_service(GlJournalEntryServiceServer::new(gl_service))
        .serve_with_shutdown(addr, async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install CTRL+C signal handler");
            info!("Shutting down gl-service...");
        })
        .await?;
    
    info!("gl-service graceful shutdown complete");
    Ok(())
}
