use tonic::transport::Server;
use tracing::info;
use std::sync::Arc;
use so_service::api::grpc_server::SoServiceImpl;
use so_service::api::proto::sd::so::v1::sales_order_fulfillment_service_server::SalesOrderFulfillmentServiceServer;
use so_service::infrastructure::repository::SalesOrderRepository;
use so_service::application::handlers::{CreateSalesOrderHandler, GetSalesOrderHandler, ListSalesOrdersHandler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50055).await?;
    let pool = context.db_pool;
    let addr = context.addr;

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
