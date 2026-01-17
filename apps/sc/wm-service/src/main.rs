use tonic::transport::Server;
use tracing::info;
use std::sync::Arc;

use wm_service::api::grpc_server::WmServiceImpl;
use wm_service::api::proto::sc::wm::v1::warehouse_operations_service_server::WarehouseOperationsServiceServer;
use wm_service::infrastructure::repository::TransferOrderRepository;
use wm_service::application::handlers::WarehouseHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50070).await?;
    let pool = context.db_pool;
    let addr = context.addr;

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
