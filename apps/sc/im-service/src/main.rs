use tonic::transport::Server;
use tracing::info;
use dotenvy::dotenv;
use std::sync::Arc;
use cuba_database::{DatabaseConfig, init_pool};

use im_service::api::grpc_server::ImServiceImpl;
use im_service::api::proto::sc::im::v1::inventory_management_service_server::InventoryManagementServiceServer;
use im_service::infrastructure::repository::InventoryRepository;
use im_service::application::handlers::{PostStockMovementHandler, GetStockOverviewHandler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    dotenv().ok();
    
    // Port 50056
    let addr = "0.0.0.0:50056".parse()?;
    info!("Starting SC Inventory Management Service on {}", addr);

    // Database
    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

    // Run migrations
    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    // Infrastructure
    let repo = Arc::new(InventoryRepository::new(pool.clone()));
    
    // Application Handlers
    let post_handler = Arc::new(PostStockMovementHandler::new(repo.clone()));
    let get_stock_handler = Arc::new(GetStockOverviewHandler::new(repo.clone()));
    
    // API
    let service = ImServiceImpl::new(
        post_handler,
        get_stock_handler,
    );
    
    // Reflection Service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(im_service::api::proto::sc::im::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("SC Inventory Management Service listening on {}", addr);
    
    Server::builder()
        .add_service(InventoryManagementServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
