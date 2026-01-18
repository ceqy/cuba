use sqlx::PgPool;
use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;
use uj_service::{
    api::UniversalJournalServiceImpl,
    infrastructure::{
        grpc::proto::uj::v1::universal_journal_service_server::UniversalJournalServiceServer,
        persistence::PostgresUniversalJournalRepository,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::INFO)
            .finish(),
    )
    .expect("Failed to set tracing subscriber");

    info!("Starting Universal Journal Service...");

    // 加载配置
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/erp".to_string());

    // 连接数据库
    info!("Connecting to database: {}", database_url);
    let pool = PgPool::connect(&database_url).await?;

    // 运行数据库迁移
    info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;

    // 创建仓储
    let repository = Arc::new(PostgresUniversalJournalRepository::new(pool));

    // 创建 gRPC 服务
    let uj_service = UniversalJournalServiceImpl::new(repository);

    // 启动 gRPC 服务器
    let addr = "0.0.0.0:50055".parse()?;
    info!("Universal Journal Service listening on {}", addr);

    Server::builder()
        .add_service(UniversalJournalServiceServer::new(uj_service))
        .serve(addr)
        .await?;

    Ok(())
}
