use tonic::transport::Server;
use tracing::info;
use std::sync::Arc;

use im_service::api::grpc_server::ImServiceImpl;
use im_service::api::proto::sc::im::v1::inventory_management_service_server::InventoryManagementServiceServer;
use im_service::infrastructure::repository::InventoryRepository;
use im_service::application::handlers::{PostStockMovementHandler, GetStockOverviewHandler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50056).await?;
    let pool = context.db_pool;
    let addr = context.addr;

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
