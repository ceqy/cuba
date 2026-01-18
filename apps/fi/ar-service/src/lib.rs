pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;

// Re-export for convenience
pub use api::grpc_server::ArServiceImpl;
pub use infrastructure::repository::{CustomerRepository, OpenItemRepository, InvoiceRepository};

use std::sync::Arc;
use tokio::sync::Mutex;
use cuba_database::DbPool;
use cuba_finance::GlClient;

/// Factory function to create and wire up the AR Service with all its dependencies.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `gl_client` - GL service client for journal entry creation
///
/// # Returns
/// Fully initialized ArServiceImpl ready to serve gRPC requests
pub fn create_ar_service(
    pool: DbPool,
    gl_client: Arc<Mutex<GlClient>>,
) -> ArServiceImpl {
    use application::handlers::*;

    // Initialize Repositories
    let customer_repo = Arc::new(CustomerRepository::new(pool.clone()));
    let open_item_repo = Arc::new(OpenItemRepository::new(pool.clone()));
    let invoice_repo = Arc::new(InvoiceRepository::new(pool.clone()));

    // Initialize Handlers
    let post_customer_handler = Arc::new(PostCustomerHandler::new(customer_repo.clone()));
    let list_open_items_handler = Arc::new(ListOpenItemsHandler::new(open_item_repo.clone()));
    let post_sales_invoice_handler = Arc::new(PostSalesInvoiceHandler::new(
        customer_repo.clone(),
        invoice_repo.clone(),
        gl_client.clone(),
    ));
    let clear_open_items_handler = Arc::new(ClearOpenItemsHandler::new(open_item_repo.clone()));
    let partial_clear_handler = Arc::new(PartialClearHandler::new(open_item_repo.clone()));

    // Assemble Service
    ArServiceImpl::new(
        post_customer_handler,
        list_open_items_handler,
        post_sales_invoice_handler,
        clear_open_items_handler,
        partial_clear_handler,
    )
}
