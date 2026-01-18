use crate::domain::{Customer, OpenItem, Invoice, InvoiceItem, InvoiceStatus};
use crate::infrastructure::repository::{CustomerRepository, OpenItemRepository, InvoiceRepository};
use crate::application::commands::{PostCustomerCommand, ListOpenItemsQuery, PostSalesInvoiceCommand};
use std::sync::Arc;
use chrono::{Utc, Datelike};
use rust_decimal::Decimal;
use uuid::Uuid;

pub struct PostCustomerHandler {
    repo: Arc<CustomerRepository>,
}

impl PostCustomerHandler {
    pub fn new(repo: Arc<CustomerRepository>) -> Self {
        Self { repo }
    }

    pub async fn handle(&self, cmd: PostCustomerCommand) -> anyhow::Result<Customer> {
        let customer = Customer {
            customer_id: cmd.customer_id,
            business_partner_id: cmd.business_partner_id,
            name: cmd.name,
            account_group: cmd.account_group,
            street: cmd.street,
            city: cmd.city,
            postal_code: cmd.postal_code,
            country: cmd.country,
            company_code: cmd.company_code,
            reconciliation_account: cmd.reconciliation_account,
            payment_terms: cmd.payment_terms,
            sales_organization: cmd.sales_organization,
            distribution_channel: None,
            division: None,
            order_currency: cmd.order_currency,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.repo.save(&customer).await?;
        Ok(customer)
    }
}

pub struct ListOpenItemsHandler {
    repo: Arc<OpenItemRepository>,
}

impl ListOpenItemsHandler {
    pub fn new(repo: Arc<OpenItemRepository>) -> Self {
        Self { repo }
    }

    pub async fn handle(&self, query: ListOpenItemsQuery) -> anyhow::Result<Vec<OpenItem>> {
        let page = query.page_token.as_ref().and_then(|t| t.parse::<i64>().ok()).unwrap_or(1);
        let limit = query.page_size as i64;
        let offset = (page - 1) * limit;

        self.repo.list_by_customer(&query.customer_id, query.include_cleared, limit, offset).await
    }
}

/// Handler for posting sales invoices (AR)
pub struct PostSalesInvoiceHandler {
    customer_repo: Arc<CustomerRepository>,
    invoice_repo: Arc<InvoiceRepository>,
    gl_client: Arc<tokio::sync::Mutex<cuba_finance::GlClient>>,
}

impl PostSalesInvoiceHandler {
    pub fn new(
        customer_repo: Arc<CustomerRepository>,
        invoice_repo: Arc<InvoiceRepository>,
        gl_client: Arc<tokio::sync::Mutex<cuba_finance::GlClient>>,
    ) -> Self {
        Self { customer_repo, invoice_repo, gl_client }
    }

    pub async fn handle(&self, cmd: PostSalesInvoiceCommand) -> anyhow::Result<Invoice> {
        // 1. Validate Customer exists
        let _customer = self.customer_repo.find_by_customer_id(&cmd.customer_id).await?
            .ok_or_else(|| anyhow::anyhow!("Customer not found: {}", cmd.customer_id))?;

        // 2. Build Invoice Items
        let invoice_id = Uuid::new_v4();
        let mut total_amount = Decimal::ZERO;
        let items: Vec<InvoiceItem> = cmd.items.iter().enumerate().map(|(i, item)| {
            total_amount += item.amount;
            InvoiceItem {
                item_id: Uuid::new_v4(),
                line_item_number: (i + 1) as i32,
                description: item.item_text.clone(),
                quantity: None,
                unit_price: None,
                total_price: item.amount,
                gl_account: item.gl_account.clone(),
                tax_code: None,
                profit_center: item.cost_center.clone(),
            }
        }).collect();

        // 3. Create Invoice Aggregate
        let now = Utc::now();
        let invoice = Invoice {
            invoice_id,
            document_number: Some(format!("DR-{}-{}", cmd.document_date.year(), Uuid::new_v4().simple().to_string().chars().take(8).collect::<String>())),
            company_code: cmd.company_code.clone(),
            fiscal_year: cmd.document_date.year(),
            document_date: cmd.document_date,
            posting_date: cmd.posting_date,
            customer_id: cmd.customer_id.clone(),
            currency: cmd.currency.clone(),
            total_amount,
            reference: cmd.reference_document.clone(),
            status: InvoiceStatus::Posted,
            items: items.clone(),
            created_at: now,
            updated_at: now,
        };

        // 4. Save Invoice
        self.invoice_repo.save(&invoice).await?;

        // 5. Integrate with GL - Create Journal Entry
        // AR Invoice: Debit Receivables, Credit Revenue
        let gl_line_items: Vec<cuba_finance::GlLineItem> = cmd.items.iter().map(|item| {
            cuba_finance::GlLineItem {
                gl_account: item.gl_account.clone(),
                debit_credit: item.debit_credit.clone(),
                amount: item.amount,
                cost_center: item.cost_center.clone(),
                profit_center: None,
                item_text: item.item_text.clone(),
                business_partner: Some(cmd.customer_id.clone()),
                special_gl_indicator: None, // 普通业务，特殊业务需要单独处理
                ledger: cmd.ledger.clone(),
                ledger_type: cmd.ledger_type,
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
            invoice.reference.clone(),
            None,
            gl_line_items,
            None, // 使用默认主分类账 "0L"
        ).await {
            Ok(response) => {
                tracing::info!(
                    "GL Journal Entry created for AR invoice {:?}: {:?}",
                    invoice.document_number,
                    response.document_reference
                );
            }
            Err(e) => {
                tracing::error!("Failed to create GL entry for AR invoice {:?}: {}", invoice.document_number, e);
            }
        }

        Ok(invoice)
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
    ) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let clearing_date = chrono::Utc::now().naive_utc().date();

        let cleared_count = self.open_item_repo
            .clear_items(&item_ids, &clearing_document, clearing_date)
            .await?;

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
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let clearing_date = chrono::Utc::now().naive_utc().date();

        self.open_item_repo
            .partial_clear(item_id, amount_to_clear, &clearing_document, clearing_date)
            .await?;

        Ok(())
    }
}

