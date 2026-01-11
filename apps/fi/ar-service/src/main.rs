use tonic::transport::Server;
use tracing::info;
use dotenvy::dotenv;
use std::sync::Arc;
use cuba_database::{DatabaseConfig, init_pool};

use ar_service::api::grpc_server::ArServiceImpl;
use ar_service::api::proto::fi::ap::v1::accounts_receivable_payable_service_server::AccountsReceivablePayableServiceServer;
use ar_service::infrastructure::repository::{CustomerRepository, OpenItemRepository};
use ar_service::application::handlers::{PostCustomerHandler, ListOpenItemsHandler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    dotenv().ok();
    
    // Assign a default port for AR, e.g. 50054 (AP is 53)
    let addr = "0.0.0.0:50054".parse()?;
    info!("Starting AR Service on {}", addr);

    // Database
    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

    // Run migrations
    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    // Infrastructure
    let customer_repo = Arc::new(CustomerRepository::new(pool.clone()));
    let open_item_repo = Arc::new(OpenItemRepository::new(pool.clone()));
    
    // Application Handlers
    let post_customer_handler = Arc::new(PostCustomerHandler::new(customer_repo.clone()));
    let list_open_items_handler = Arc::new(ListOpenItemsHandler::new(open_item_repo.clone()));
    
    // API
    let ar_service = ArServiceImpl::new(
        post_customer_handler,
        list_open_items_handler,
    );
    
    // Reflection Service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(ar_service::api::proto::fi::ap::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("AR Service listening on {}", addr);
    
    Server::builder()
        .add_service(AccountsReceivablePayableServiceServer::new(ar_service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
