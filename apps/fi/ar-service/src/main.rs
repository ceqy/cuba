use tonic::transport::Server;
use tracing::info;
use std::sync::Arc;
use tokio::sync::Mutex;

use ar_service::api::grpc_server::ArServiceImpl;
use ar_service::api::proto::fi::ap::v1::accounts_receivable_payable_service_server::AccountsReceivablePayableServiceServer;
use ar_service::infrastructure::repository::{CustomerRepository, OpenItemRepository, InvoiceRepository};
use ar_service::infrastructure::gl_client::GlClient;
use ar_service::application::handlers::{PostCustomerHandler, ListOpenItemsHandler, PostSalesInvoiceHandler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50062).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    // GL Service Endpoint (from env or default)
    let gl_endpoint = std::env::var("GL_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:50051".to_string());
    info!("GL Service endpoint: {}", gl_endpoint);

    // Run migrations
    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;

    // Run migrations
    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;

    // GL Client
    let gl_client = Arc::new(Mutex::new(
        GlClient::new(&gl_endpoint).await
            .map_err(|e| format!("Failed to connect to GL service: {}", e))?
    ));
    
    // Infrastructure
    let customer_repo = Arc::new(CustomerRepository::new(pool.clone()));
    let open_item_repo = Arc::new(OpenItemRepository::new(pool.clone()));
    let invoice_repo = Arc::new(InvoiceRepository::new(pool.clone()));
    
    // Application Handlers
    let post_customer_handler = Arc::new(PostCustomerHandler::new(customer_repo.clone()));
    let list_open_items_handler = Arc::new(ListOpenItemsHandler::new(open_item_repo.clone()));
    let post_sales_invoice_handler = Arc::new(PostSalesInvoiceHandler::new(
        customer_repo.clone(),
        invoice_repo.clone(),
        gl_client.clone(),
    ));
    
    // API
    let ar_service = ArServiceImpl::new(
        post_customer_handler,
        list_open_items_handler,
        post_sales_invoice_handler,
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
