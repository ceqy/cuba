//! Finance Service - Main Entry Point
//!
//! CUBA Enterprise 财务服务 - 总账凭证服务

mod application;
mod domain;
mod grpc;
mod infrastructure;
mod proto;

use grpc::GlJournalEntryServiceImpl;
use infrastructure::metrics::GlServiceMetrics;
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
    #[serde(default = "default_metrics_port")]
    metrics_port: u16,
}

fn default_metrics_port() -> u16 {
    9200
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
    
    if let Ok(db_url) = std::env::var("GL_DATABASE_URL") {
        config.database_url = db_url;
    } else if let Ok(db_url) = std::env::var("DATABASE_URL") {
        config.database_url = db_url;
    }

    if let Ok(port) = std::env::var("METRICS_PORT") {
        if let Ok(p) = port.parse() {
            config.metrics_port = p;
        }
    }
    
    info!("Starting gl-service on {}", config.server_addr);

    // 创建指标注册表
    let metrics = GlServiceMetrics::new();
    let metrics_clone = metrics.clone();

    // 创建数据库连接池
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;
    let pool = Arc::new(pool);

    // 运行数据库迁移
    info!("Running database migrations for cuba_finance_gl...");
    sqlx::migrate!("./migrations")
        .run(&*pool)
        .await?;
    info!("Database migrations completed");

    // 初始化仓储和应用服务
    let repository = Arc::new(crate::infrastructure::persistence::PgJournalEntryRepository::new(pool.clone()));
    let journal_service = Arc::new(crate::application::JournalEntryService::new(repository));

    // 创建 gRPC 服务
    let gl_service = GlJournalEntryServiceImpl::new(journal_service);

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

    // 启动 HTTP 指标服务器 (单独线程)
    let metrics_addr = format!("0.0.0.0:{}", config.metrics_port);
    info!("Starting metrics server on {}", metrics_addr);
    
    tokio::spawn(async move {
        start_metrics_server(metrics_clone, &metrics_addr).await;
    });

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

/// 启动简易 HTTP 服务器提供 /metrics 端点
async fn start_metrics_server(metrics: Arc<GlServiceMetrics>, addr: &str) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("Failed to bind metrics server: {}", e);
            return;
        }
    };

    loop {
        if let Ok((mut socket, _)) = listener.accept().await {
            let metrics = metrics.clone();
            tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                if socket.read(&mut buf).await.is_ok() {
                    let request = String::from_utf8_lossy(&buf);
                    
                    let response = if request.starts_with("GET /metrics") {
                        let body = metrics.to_prometheus();
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                            body.len(),
                            body
                        )
                    } else if request.starts_with("GET /health") {
                        let body = r#"{"status":"healthy"}"#;
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                            body.len(),
                            body
                        )
                    } else {
                        "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_string()
                    };

                    let _ = socket.write_all(response.as_bytes()).await;
                }
            });
        }
    }
}
