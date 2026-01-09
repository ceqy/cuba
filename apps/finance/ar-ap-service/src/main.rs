//! AR/AP Service - Main Entry Point
//!
//! CUBA Enterprise 财务服务 - 应收应付服务

mod application;
mod domain;
mod grpc;
mod infrastructure;
mod proto;

use grpc::ArApServiceImpl;
use proto::finance::arap::accounts_receivable_payable_service_server::AccountsReceivablePayableServiceServer;
use proto::finance::arap::FILE_DESCRIPTOR_SET;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    cuba_telemetry::init_subscriber("ar-ap-service");

    // 从环境变量获取配置
    let server_addr = std::env::var("SERVER_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50052".to_string());
    
    let database_url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("ARAP_DATABASE_URL"))
        .unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost/cuba_finance".to_string()
        });

    info!("Starting ar-ap-service on {}", server_addr);

    // 创建数据库连接池
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;
    let pool = Arc::new(pool);

    // 运行数据库迁移
    info!("Running database migrations for cuba_finance_arap...");
    sqlx::migrate!("./migrations")
        .run(&*pool)
        .await?;
    info!("Database migrations completed");

    // 初始化仓储
    let repository = Arc::new(infrastructure::PgArApRepository::new(pool.clone()));

    // 创建 gRPC 服务
    let arap_service = ArApServiceImpl::new(repository);

    // --- gRPC 反射服务 ---
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()?;

    // --- gRPC 健康检查服务 ---
    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<AccountsReceivablePayableServiceServer<ArApServiceImpl>>()
        .await;

    // 解析地址
    let addr: std::net::SocketAddr = server_addr.parse()?;

    info!("AR/AP service listening on {}", addr);

    // 启动 gRPC 服务器 (带优雅停机)
    tonic::transport::Server::builder()
        .add_service(health_service)
        .add_service(reflection_service)
        .add_service(AccountsReceivablePayableServiceServer::new(arap_service))
        .serve_with_shutdown(addr, async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install CTRL+C signal handler");
            info!("Shutting down ar-ap-service...");
        })
        .await?;

    info!("ar-ap-service graceful shutdown complete");
    Ok(())
}
