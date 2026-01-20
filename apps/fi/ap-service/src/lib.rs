pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;

// Re-export for convenience
pub use api::grpc_server::ApServiceImpl;
pub use infrastructure::repository::{InvoiceRepository, OpenItemRepository, SupplierRepository};

use cuba_database::DbPool;
use cuba_finance::GlClient;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Factory function to create and wire up the AP Service with all its dependencies.
///
/// This encapsulates the complex initialization logic and makes main.rs cleaner.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `gl_client` - GL service client for journal entry creation
///
/// # Returns
/// Fully initialized ApServiceImpl ready to serve gRPC requests
pub fn create_ap_service(pool: DbPool, gl_client: Arc<Mutex<GlClient>>) -> ApServiceImpl {
    use application::handlers::*;

    // Initialize Repositories
    let supplier_repo = Arc::new(SupplierRepository::new(pool.clone()));
    let open_item_repo = Arc::new(OpenItemRepository::new(pool.clone()));
    let invoice_repo = Arc::new(InvoiceRepository::new(pool.clone()));

    // Initialize Handlers
    let post_supplier_handler = Arc::new(PostSupplierHandler::new(supplier_repo.clone()));
    let list_open_items_handler = Arc::new(ListOpenItemsHandler::new(
        supplier_repo.clone(),
        open_item_repo.clone(),
    ));
    let post_invoice_handler = Arc::new(PostInvoiceHandler::new(
        invoice_repo.clone(),
        supplier_repo.clone(),
        gl_client.clone(),
    ));
    let get_invoice_handler = Arc::new(GetInvoiceHandler::new(invoice_repo.clone()));
    let approve_invoice_handler = Arc::new(ApproveInvoiceHandler::new(invoice_repo.clone()));
    let reject_invoice_handler = Arc::new(RejectInvoiceHandler::new(invoice_repo.clone()));
    let clear_open_items_handler = Arc::new(ClearOpenItemsHandler::new(open_item_repo.clone()));
    let partial_clear_handler = Arc::new(PartialClearHandler::new(open_item_repo.clone()));
    let generate_payment_proposal_handler =
        Arc::new(GeneratePaymentProposalHandler::new(open_item_repo.clone()));
    let execute_payment_proposal_handler =
        Arc::new(ExecutePaymentProposalHandler::new(open_item_repo.clone()));

    // Assemble Service
    ApServiceImpl::new(
        post_supplier_handler,
        list_open_items_handler,
        post_invoice_handler,
        get_invoice_handler,
        approve_invoice_handler,
        reject_invoice_handler,
        clear_open_items_handler,
        partial_clear_handler,
        generate_payment_proposal_handler,
        execute_payment_proposal_handler,
    )
}
