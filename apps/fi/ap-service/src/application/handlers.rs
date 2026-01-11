//! AP Service Handlers
//! Business logic handlers for commands and queries

use std::sync::Arc;
use uuid::Uuid;
use chrono::{Utc, Datelike};
use rust_decimal::Decimal;

use crate::application::commands::{PostSupplierCommand, PostInvoiceCommand, ListOpenItemsQuery};
use crate::domain::{Supplier, Invoice, InvoiceItem, InvoiceStatus, OpenItem, DebitCredit};
use crate::infrastructure::repository::{SupplierRepository, OpenItemRepository};

/// Handler for posting suppliers
pub struct PostSupplierHandler {
    repo: Arc<SupplierRepository>,
}

impl PostSupplierHandler {
    pub fn new(repo: Arc<SupplierRepository>) -> Self {
        Self { repo }
    }

    pub async fn handle(&self, cmd: PostSupplierCommand) -> Result<Supplier, AppError> {
        let now = Utc::now();
        
        let supplier = Supplier {
            id: Uuid::new_v4(),
            supplier_id: cmd.supplier_id,
            business_partner_id: cmd.business_partner_id,
            name: cmd.name,
            account_group: cmd.account_group,
            street: cmd.street,
            city: cmd.city,
            postal_code: cmd.postal_code,
            country: cmd.country,
            telephone: cmd.telephone,
            email: cmd.email,
            company_code: cmd.company_code,
            reconciliation_account: cmd.reconciliation_account,
            payment_terms: cmd.payment_terms,
            check_double_invoice: cmd.check_double_invoice,
            purchasing_organization: cmd.purchasing_organization,
            order_currency: cmd.order_currency.unwrap_or_else(|| "CNY".to_string()),
            created_at: now,
            updated_at: now,
        };

        self.repo.save(&supplier).await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(supplier)
    }
}

/// Handler for listing open items
pub struct ListOpenItemsHandler {
    supplier_repo: Arc<SupplierRepository>,
    open_item_repo: Arc<OpenItemRepository>,
}

impl ListOpenItemsHandler {
    pub fn new(
        supplier_repo: Arc<SupplierRepository>,
        open_item_repo: Arc<OpenItemRepository>,
    ) -> Self {
        Self { supplier_repo, open_item_repo }
    }

    pub async fn handle(&self, query: ListOpenItemsQuery) -> Result<Vec<OpenItem>, AppError> {
        // Find supplier by business partner ID
        let supplier = self.supplier_repo
            .find_by_supplier_id(&query.business_partner_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Supplier not found".to_string()))?;

        // List open items
        let items = self.open_item_repo
            .list_by_supplier(supplier.id, &query.company_code, query.include_cleared)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(items)
    }
}

/// Application error types
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Business rule violation: {0}")]
    BusinessRule(String),
}

impl From<AppError> for tonic::Status {
    fn from(err: AppError) -> Self {
        match err {
            AppError::Database(msg) => tonic::Status::internal(msg),
            AppError::NotFound(msg) => tonic::Status::not_found(msg),
            AppError::Validation(msg) => tonic::Status::invalid_argument(msg),
            AppError::BusinessRule(msg) => tonic::Status::failed_precondition(msg),
        }
    }
}
