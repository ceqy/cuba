use tonic::transport::Server;
use tracing::info;
use dotenvy::dotenv;
use std::sync::Arc;
use cuba_database::{DatabaseConfig, init_pool};

use wm_service::api::grpc_server::WmServiceImpl;
use wm_service::api::proto::sc::wm::v1::warehouse_operations_service_server::WarehouseOperationsServiceServer;
use wm_service::infrastructure::repository::TransferOrderRepository;
use wm_service::application::handlers::WarehouseHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    dotenv().ok();
    
    let addr = "0.0.0.0:50070".parse()?;
    info!("Starting SC Warehouse Operations Service on {}", addr);

    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    let repo = Arc::new(TransferOrderRepository::new(pool.clone()));
    let handler = Arc::new(WarehouseHandler::new(repo.clone()));
    let service = WmServiceImpl::new(handler, repo);
    
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(wm_service::api::proto::sc::wm::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("SC Warehouse Operations Service listening on {}", addr);
    
    Server::builder()
        .add_service(WarehouseOperationsServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
