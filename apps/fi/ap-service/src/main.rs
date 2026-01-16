use tonic::transport::Server;
use cuba_database::{DatabaseConfig, init_pool};
use tracing::info;
use std::sync::Arc;
use dotenvy::dotenv;
use tokio::sync::Mutex;

use ap_service::api::grpc_server::ApServiceImpl;
use ap_service::api::proto::fi::ap::v1::accounts_receivable_payable_service_server::AccountsReceivablePayableServiceServer;
use ap_service::infrastructure::repository::{SupplierRepository, OpenItemRepository, InvoiceRepository};
use ap_service::infrastructure::gl_client::GlClient;
use ap_service::application::handlers::{PostSupplierHandler, ListOpenItemsHandler, PostInvoiceHandler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok(); // Load .env
    cuba_telemetry::init_telemetry();
    
    // Config
    let addr = "0.0.0.0:50053".parse()?;
    info!("Starting ap-service on {}", addr);

    // GL Service Endpoint (from env or default)
    let gl_endpoint = std::env::var("GL_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:50051".to_string());
    info!("GL Service endpoint: {}", gl_endpoint);

    // Database
    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;
    info!("Connected to database: {}", db_config.url);

    // GL Client
    let gl_client = Arc::new(Mutex::new(
        GlClient::new(&gl_endpoint).await
            .map_err(|e| format!("Failed to connect to GL service: {}", e))?
    ));

    // Repositories
    let supplier_repo = Arc::new(SupplierRepository::new(pool.clone()));
    let open_item_repo = Arc::new(OpenItemRepository::new(pool.clone()));
    let invoice_repo = Arc::new(InvoiceRepository::new(pool.clone()));

    // Handlers
    let post_supplier_handler = Arc::new(PostSupplierHandler::new(supplier_repo.clone()));
    let list_open_items_handler = Arc::new(ListOpenItemsHandler::new(supplier_repo.clone(), open_item_repo.clone()));
    let post_invoice_handler = Arc::new(PostInvoiceHandler::new(
        invoice_repo.clone(),
        supplier_repo.clone(),
        gl_client.clone(),
    ));

    // Service
    let ap_service = ApServiceImpl::new(
        post_supplier_handler,
        list_open_items_handler,
        post_invoice_handler,
    );

    // Reflection
    let descriptor = include_bytes!(concat!(env!("OUT_DIR"), "/descriptor.bin"));
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(descriptor)
        .build_v1()?;

    info!("AP Service listening on {}", addr);
    
    Server::builder()
        .add_service(reflection_service)
        .add_service(AccountsReceivablePayableServiceServer::new(ap_service))
        .serve(addr)
        .await?;

    Ok(())
}
