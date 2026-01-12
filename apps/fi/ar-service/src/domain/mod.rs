use rust_decimal::Decimal;
use chrono::{NaiveDate, DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Enums
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebitCredit {
    Debit,  // S
    Credit, // H
}

impl DebitCredit {
    pub fn as_char(&self) -> char {
        match self {
            DebitCredit::Debit => 'S',
            DebitCredit::Credit => 'H',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvoiceStatus {
    Draft,
    Posted,
    Cancelled,
}

impl Default for InvoiceStatus {
    fn default() -> Self {
        InvoiceStatus::Draft
    }
}

// ============================================================================
// Aggregates & Entities
// ============================================================================

/// Customer Master Data (Aggregate Root)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Customer {
    pub customer_id: String,
    pub business_partner_id: Option<String>,
    pub name: String,
    pub account_group: String,
    
    // Address
    pub street: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    
    // Control
    pub company_code: String,
    pub reconciliation_account: String,
    pub payment_terms: Option<String>,
    
    // Sales
    pub sales_organization: Option<String>,
    pub distribution_channel: Option<String>,
    pub division: Option<String>,
    pub order_currency: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sales Invoice (Aggregate Root)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Invoice {
    pub invoice_id: Uuid,
    pub document_number: Option<String>,
    pub company_code: String,
    pub fiscal_year: i32,
    pub document_date: NaiveDate,
    pub posting_date: NaiveDate,
    
    pub customer_id: String,
    pub currency: String,
    pub total_amount: Decimal,
    
    pub reference: Option<String>,
    pub status: InvoiceStatus,
    
    #[sqlx(skip)]
    pub items: Vec<InvoiceItem>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct InvoiceItem {
    pub item_id: Uuid,
    pub line_item_number: i32,
    pub description: Option<String>,
    pub quantity: Option<Decimal>,
    pub unit_price: Option<Decimal>,
    pub total_price: Decimal,
    pub gl_account: String,
    pub tax_code: Option<String>,
    pub profit_center: Option<String>,
}

/// Open Item (Receivable) - Projection/Read Model mostly, but can be an entity for clearing
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OpenItem {
    pub open_item_id: Uuid,
    
    // Document Key
    pub document_number: String,
    pub fiscal_year: i32,
    pub company_code: String,
    pub line_item_number: i32,
    
    pub customer_id: String,
    pub doc_type: String,
    pub posting_date: NaiveDate,
    pub due_date: NaiveDate,
    
    pub currency: String,
    pub original_amount: Decimal,
    pub open_amount: Decimal,
    
    pub is_cleared: bool,
    pub payment_block: Option<String>,
    pub reference_document: Option<String>,
    pub item_text: Option<String>,
}
