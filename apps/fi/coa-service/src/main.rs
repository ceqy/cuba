// COA Service Main Entry Point
use std::sync::Arc;
use tonic::transport::Server;

use coa_service::api::grpc_server::{proto::chart_of_accounts_service_server::ChartOfAccountsServiceServer, CoaGrpcService};
use coa_service::application::CoaApplicationService;
use coa_service::infrastructure::PgGlAccountRepository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    tracing::info!("Starting COA Service...");

    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/coa".to_string());

    let pool = sqlx::PgPool::connect(&database_url).await?;
    tracing::info!("Connected to database");

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Migrations completed");

    // Initialize layers
    let repository = Arc::new(PgGlAccountRepository::new(pool.clone()));
    let app_service = Arc::new(CoaApplicationService::new(repository));
    let grpc_service = CoaGrpcService::new(app_service);

    // gRPC server address
    let addr = "0.0.0.0:50060".parse()?;

    tracing::info!("COA Service listening on {}", addr);

    // Start gRPC server
    Server::builder()
        .add_service(ChartOfAccountsServiceServer::new(grpc_service))
        .serve(addr)
        .await?;

    Ok(())
}
