use tonic::transport::Server;
use tracing::info;
use dotenvy::dotenv;
use std::sync::Arc;
use cuba_database::{DatabaseConfig, init_pool};

use co_service::api::grpc_server::CoServiceImpl;
use co_service::api::proto::fi::co::v1::controlling_allocation_service_server::ControllingAllocationServiceServer;
use co_service::infrastructure::repository::AllocationRepository;
use co_service::application::handlers::AllocationHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    dotenv().ok();
    
    let addr = "0.0.0.0:50067".parse()?;
    info!("Starting FI Controlling Allocation Service on {}", addr);

    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    let repo = Arc::new(AllocationRepository::new(pool.clone()));
    let handler = Arc::new(AllocationHandler::new(repo.clone()));
    let service = CoServiceImpl::new(handler, repo);
    
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(co_service::api::proto::fi::co::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("FI Controlling Allocation Service listening on {}", addr);
    
    Server::builder()
        .add_service(ControllingAllocationServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
