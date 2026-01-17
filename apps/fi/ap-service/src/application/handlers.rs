//! AP Service Handlers
//! Business logic handlers for commands and queries

use std::sync::Arc;
use uuid::Uuid;
use chrono::{Utc, Datelike};
use rust_decimal::Decimal;
use tokio::sync::Mutex;

use crate::application::commands::{PostSupplierCommand, PostInvoiceCommand, ListOpenItemsQuery};
use crate::domain::{Supplier, Invoice, InvoiceItem, OpenItem};
use crate::infrastructure::repository::{InvoiceRepository, SupplierRepository, OpenItemRepository};
use crate::infrastructure::gl_client::{GlClient, GlLineItem};


/// Handler for posting invoices
pub struct PostInvoiceHandler {
    invoice_repo: Arc<InvoiceRepository>,
    supplier_repo: Arc<SupplierRepository>,
    gl_client: Arc<Mutex<GlClient>>,
}

impl PostInvoiceHandler {
    pub fn new(
        invoice_repo: Arc<InvoiceRepository>,
        supplier_repo: Arc<SupplierRepository>,
        gl_client: Arc<Mutex<GlClient>>,
    ) -> Self {
        Self { invoice_repo, supplier_repo, gl_client }
    }

    pub async fn handle(&self, cmd: PostInvoiceCommand) -> Result<Invoice, AppError> {
        let now = Utc::now();
        
        // 1. Validate Supplier
        let supplier_uuid = Uuid::parse_str(&cmd.supplier_id)
            .map_err(|_| AppError::Validation("Invalid Supplier ID format".to_string()))?;
            
        let _supplier = self.supplier_repo.find_by_id(supplier_uuid).await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Supplier not found".to_string()))?;

        // 2. Map Items
        let invoice_id = Uuid::new_v4();
        let mut total_amount = Decimal::ZERO;
        
        let items: Vec<InvoiceItem> = cmd.items.iter().enumerate().map(|(i, item)| {
            total_amount += item.amount;
            InvoiceItem {
                id: Uuid::new_v4(),
                invoice_id,
                line_item_number: (i + 1) as i32,
                gl_account: item.gl_account.clone(),
                debit_credit_indicator: match item.debit_credit.as_str() {
                    "H" | "Credit" => "H".to_string(),
                    _ => "S".to_string(),
                },
                amount: item.amount,
                cost_center: item.cost_center.clone(),
                profit_center: None,
                item_text: item.item_text.clone(),
                purchase_order: item.purchase_order.clone(),
                po_item_number: item.po_item_number,
                goods_receipt: None,
                gr_item_number: None,
                quantity: None,
                unit_of_measure: None,
            }
        }).collect();

        // 3. Create Invoice Aggregate
        let invoice = Invoice {
            id: invoice_id,
            document_number: format!("INV-{}-{}", cmd.document_date.year(), Uuid::new_v4().simple().to_string().chars().take(8).collect::<String>()),
            company_code: cmd.company_code.clone(),
            fiscal_year: cmd.document_date.year(),
            document_type: "KR".to_string(),
            supplier_id: supplier_uuid,
            document_date: cmd.document_date,
            posting_date: cmd.posting_date,
            due_date: cmd.posting_date,
            baseline_date: Some(cmd.posting_date),
            currency: cmd.currency.clone(),
            total_amount,
            tax_amount: Decimal::ZERO,
            reference_document: cmd.reference_document.clone(),
            header_text: cmd.header_text.clone(),
            status: "OPEN".to_string(),
            clearing_document: None,
            clearing_date: None,
            items: items.clone(),
            created_at: now,
            updated_at: now,
        };

        // 4. Save Invoice
        self.invoice_repo.save(&invoice).await
            .map_err(|e: sqlx::Error| AppError::Database(e.to_string()))?;

        // 5. Integrate with GL - Create Journal Entry
        // AP Invoice: Debit expense accounts, Credit AP liability account
        let gl_line_items: Vec<GlLineItem> = cmd.items.iter().map(|item| {
            GlLineItem {
                gl_account: item.gl_account.clone(),
                debit_credit: item.debit_credit.clone(),
                amount: item.amount,
                cost_center: item.cost_center.clone(),
                profit_center: None,
                item_text: item.item_text.clone(),
                business_partner: Some(cmd.supplier_id.clone()),
            }
        }).collect();

        // Call GL service to create journal entry
        let mut gl_client = self.gl_client.lock().await;
        match gl_client.create_invoice_journal_entry(
            &invoice.company_code,
            invoice.document_date,
            invoice.posting_date,
            invoice.fiscal_year,
            &invoice.currency,
            invoice.reference_document.clone(),
            invoice.header_text.clone(),
            gl_line_items,
        ).await {
            Ok(response) => {
                tracing::info!(
                    "GL Journal Entry created for invoice {}: {:?}",
                    invoice.document_number,
                    response.document_reference
                );
            }
            Err(e) => {
                // Log error but don't fail the whole operation (eventual consistency)
                tracing::error!("Failed to create GL entry for invoice {}: {}", invoice.document_number, e);
            }
        }

        Ok(invoice)
    }
}


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

/// Handler for getting invoice details
pub struct GetInvoiceHandler {
    invoice_repo: Arc<InvoiceRepository>,
}

impl GetInvoiceHandler {
    pub fn new(invoice_repo: Arc<InvoiceRepository>) -> Self {
        Self { invoice_repo }
    }

    pub async fn handle(&self, id: Uuid) -> Result<Invoice, AppError> {
        self.invoice_repo
            .find_by_id(id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Invoice not found".to_string()))
    }
}

/// Handler for listing invoices
pub struct ListInvoicesHandler {
    invoice_repo: Arc<InvoiceRepository>,
}

impl ListInvoicesHandler {
    pub fn new(invoice_repo: Arc<InvoiceRepository>) -> Self {
        Self { invoice_repo }
    }

    pub async fn handle(
        &self,
        company_code: &str,
        status: Option<&str>,
        page: u64,
        page_size: u64,
    ) -> Result<(Vec<Invoice>, i64), AppError> {
        let invoices = self.invoice_repo
            .list(company_code, status, page, page_size)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let total_count = self.invoice_repo
            .count(company_code, status)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok((invoices, total_count))
    }
}

/// Handler for approving invoices
pub struct ApproveInvoiceHandler {
    invoice_repo: Arc<InvoiceRepository>,
}

impl ApproveInvoiceHandler {
    pub fn new(invoice_repo: Arc<InvoiceRepository>) -> Self {
        Self { invoice_repo }
    }

    pub async fn handle(&self, id: Uuid) -> Result<(), AppError> {
        // Check invoice exists
        let _invoice = self.invoice_repo
            .find_by_id(id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Invoice not found".to_string()))?;

        // Update status to APPROVED
        self.invoice_repo
            .update_status(id, "APPROVED")
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}

/// Handler for rejecting invoices
pub struct RejectInvoiceHandler {
    invoice_repo: Arc<InvoiceRepository>,
}

impl RejectInvoiceHandler {
    pub fn new(invoice_repo: Arc<InvoiceRepository>) -> Self {
        Self { invoice_repo }
    }

    pub async fn handle(&self, id: Uuid, _reason: Option<String>) -> Result<(), AppError> {
        // Check invoice exists
        let _invoice = self.invoice_repo
            .find_by_id(id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Invoice not found".to_string()))?;

        // Update status to REJECTED
        self.invoice_repo
            .update_status(id, "REJECTED")
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // TODO: Store rejection reason

        Ok(())
    }
}

/// Handler for clearing open items
pub struct ClearOpenItemsHandler {
    open_item_repo: Arc<OpenItemRepository>,
}

impl ClearOpenItemsHandler {
    pub fn new(open_item_repo: Arc<OpenItemRepository>) -> Self {
        Self { open_item_repo }
    }

    pub async fn handle(
        &self,
        item_ids: Vec<Uuid>,
        clearing_document: String,
    ) -> Result<i64, AppError> {
        let clearing_date = chrono::Utc::now().naive_utc().date();

        let cleared_count = self.open_item_repo
            .clear_items(&item_ids, &clearing_document, clearing_date)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(cleared_count)
    }
}

/// Handler for partial clearing
pub struct PartialClearHandler {
    open_item_repo: Arc<OpenItemRepository>,
}

impl PartialClearHandler {
    pub fn new(open_item_repo: Arc<OpenItemRepository>) -> Self {
        Self { open_item_repo }
    }

    pub async fn handle(
        &self,
        item_id: Uuid,
        amount_to_clear: rust_decimal::Decimal,
        clearing_document: String,
    ) -> Result<(), AppError> {
        let clearing_date = chrono::Utc::now().naive_utc().date();

        self.open_item_repo
            .partial_clear(item_id, amount_to_clear, &clearing_document, clearing_date)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}

/// Handler for generating payment proposal
pub struct GeneratePaymentProposalHandler {
    open_item_repo: Arc<OpenItemRepository>,
}

impl GeneratePaymentProposalHandler {
    pub fn new(open_item_repo: Arc<OpenItemRepository>) -> Self {
        Self { open_item_repo }
    }

    pub async fn handle(
        &self,
        company_code: String,
        payment_date: chrono::NaiveDate,
    ) -> Result<Vec<OpenItem>, AppError> {
        // Query all open items for the company code that are due by payment_date
        let items = self.open_item_repo
            .list_due_items(&company_code, payment_date)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(items)
    }
}

/// Handler for executing payment proposal
pub struct ExecutePaymentProposalHandler {
    open_item_repo: Arc<OpenItemRepository>,
}

impl ExecutePaymentProposalHandler {
    pub fn new(open_item_repo: Arc<OpenItemRepository>) -> Self {
        Self { open_item_repo }
    }

    pub async fn handle(
        &self,
        item_ids: Vec<Uuid>,
        payment_document: String,
    ) -> Result<i64, AppError> {
        let payment_date = chrono::Utc::now().naive_utc().date();

        let cleared_count = self.open_item_repo
            .clear_items(&item_ids, &payment_document, payment_date)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(cleared_count)
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
