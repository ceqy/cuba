use tonic::transport::Server;
use tracing::info;
use dotenvy::dotenv;
use std::sync::Arc;
use cuba_database::{DatabaseConfig, init_pool};

use so_service::api::grpc_server::SoServiceImpl;
use so_service::api::proto::sd::so::v1::sales_order_fulfillment_service_server::SalesOrderFulfillmentServiceServer;
use so_service::infrastructure::repository::SalesOrderRepository;
use so_service::application::handlers::{CreateSalesOrderHandler, GetSalesOrderHandler, ListSalesOrdersHandler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    dotenv().ok();
    
    // Assign a default port for SD-SO, e.g. 50055
    let addr = "0.0.0.0:50055".parse()?;
    info!("Starting SD Sales Order Service on {}", addr);

    // Database
    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

    // Run migrations
    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    // Infrastructure
    let so_repo = Arc::new(SalesOrderRepository::new(pool.clone()));
    
    // Application Handlers
    let create_handler = Arc::new(CreateSalesOrderHandler::new(so_repo.clone()));
    let get_handler = Arc::new(GetSalesOrderHandler::new(so_repo.clone()));
    let list_handler = Arc::new(ListSalesOrdersHandler::new(so_repo.clone()));
    
    // API
    let so_service = SoServiceImpl::new(
        create_handler,
        get_handler,
        list_handler,
    );
    
    // Reflection Service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(so_service::api::proto::sd::so::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("SD Sales Order Service listening on {}", addr);
    
    Server::builder()
        .add_service(SalesOrderFulfillmentServiceServer::new(so_service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
