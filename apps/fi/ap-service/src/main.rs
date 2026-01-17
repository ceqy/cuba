use tonic::transport::Server;
use tracing::info;
use std::sync::Arc;
use tokio::sync::Mutex;


use ap_service::api::grpc_server::ApServiceImpl;
use ap_service::api::proto::fi::ap::v1::accounts_receivable_payable_service_server::AccountsReceivablePayableServiceServer;
use ap_service::infrastructure::repository::{SupplierRepository, OpenItemRepository, InvoiceRepository};
use ap_service::infrastructure::gl_client::GlClient;
use ap_service::application::handlers::{PostSupplierHandler, ListOpenItemsHandler, PostInvoiceHandler, GetInvoiceHandler, ApproveInvoiceHandler, RejectInvoiceHandler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50061).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    // GL Service Endpoint (from env or default)
    let gl_endpoint = std::env::var("GL_SERVICE_URL")
        .unwrap_or_else(|_| "http://gl-service.cuba-fi.svc.cluster.local:50060".to_string());
    info!("GL Service endpoint: {}", gl_endpoint);

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
    let get_invoice_handler = Arc::new(GetInvoiceHandler::new(invoice_repo.clone()));
    let approve_invoice_handler = Arc::new(ApproveInvoiceHandler::new(invoice_repo.clone()));
    let reject_invoice_handler = Arc::new(RejectInvoiceHandler::new(invoice_repo.clone()));

    // Service
    let ap_service = ApServiceImpl::new(
        post_supplier_handler,
        list_open_items_handler,
        post_invoice_handler,
        get_invoice_handler,
        approve_invoice_handler,
        reject_invoice_handler,
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
