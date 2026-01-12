//! AP Service Domain Models
//! Represents core business entities for Accounts Payable

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Supplier aggregate root
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Supplier {
    pub id: Uuid,
    pub supplier_id: String,
    pub business_partner_id: Option<String>,
    pub name: String,
    pub account_group: String,
    
    // Address
    pub street: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    
    // Contact
    pub telephone: Option<String>,
    pub email: Option<String>,
    
    // Company Code Data
    pub company_code: String,
    pub reconciliation_account: String,
    pub payment_terms: Option<String>,
    pub check_double_invoice: bool,
    
    // Purchasing
    pub purchasing_organization: Option<String>,
    pub order_currency: String,
    
    // Audit
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Invoice aggregate root
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Invoice {
    pub id: Uuid,
    pub document_number: String,
    pub company_code: String,
    pub fiscal_year: i32,
    pub document_type: String,
    
    // Header
    pub supplier_id: Uuid,
    pub document_date: NaiveDate,
    pub posting_date: NaiveDate,
    pub due_date: NaiveDate,
    pub baseline_date: Option<NaiveDate>,
    
    // Amounts
    pub currency: String,
    pub total_amount: Decimal,
    pub tax_amount: Decimal,
    
    // Reference
    pub reference_document: Option<String>,
    pub header_text: Option<String>,
    
    // Status
    pub status: InvoiceStatus,
    pub clearing_document: Option<String>,
    pub clearing_date: Option<NaiveDate>,
    
    // Line items
    #[sqlx(skip)]
    pub items: Vec<InvoiceItem>,
    
    // Audit
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InvoiceStatus {
    Open,
    Cleared,
    Reversed,
}

impl From<&str> for InvoiceStatus {
    fn from(s: &str) -> Self {
        match s {
            "CLEARED" => InvoiceStatus::Cleared,
            "REVERSED" => InvoiceStatus::Reversed,
            _ => InvoiceStatus::Open,
        }
    }
}

impl std::fmt::Display for InvoiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvoiceStatus::Open => write!(f, "OPEN"),
            InvoiceStatus::Cleared => write!(f, "CLEARED"),
            InvoiceStatus::Reversed => write!(f, "REVERSED"),
        }
    }
}

/// Invoice line item
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct InvoiceItem {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub line_item_number: i32,
    pub gl_account: String,
    pub debit_credit_indicator: DebitCredit,
    pub amount: Decimal,
    pub cost_center: Option<String>,
    pub profit_center: Option<String>,
    pub item_text: Option<String>,
    
    // Three-way matching
    pub purchase_order: Option<String>,
    pub po_item_number: Option<i32>,
    pub goods_receipt: Option<String>,
    pub gr_item_number: Option<i32>,
    pub quantity: Option<Decimal>,
    pub unit_of_measure: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DebitCredit {
    Debit,  // S
    Credit, // H
}

impl From<&str> for DebitCredit {
    fn from(s: &str) -> Self {
        match s {
            "H" => DebitCredit::Credit,
            _ => DebitCredit::Debit,
        }
    }
}

impl DebitCredit {
    pub fn as_str(&self) -> &str {
        match self {
            DebitCredit::Debit => "S",
            DebitCredit::Credit => "H",
        }
    }
}

/// Open Item entity
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OpenItem {
    pub id: Uuid,
    pub document_number: String,
    pub company_code: String,
    pub fiscal_year: i32,
    pub line_item_number: i32,
    pub supplier_id: Option<Uuid>,
    pub account_type: String,
    pub posting_date: NaiveDate,
    pub due_date: NaiveDate,
    pub baseline_date: Option<NaiveDate>,
    pub currency: String,
    pub original_amount: Decimal,
    pub open_amount: Decimal,
    pub is_cleared: bool,
    pub clearing_document: Option<String>,
    pub clearing_date: Option<NaiveDate>,
    pub reference_document: Option<String>,
    pub item_text: Option<String>,
    pub payment_block: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payment Document
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PaymentDocument {
    pub id: Uuid,
    pub document_number: String,
    pub company_code: String,
    pub fiscal_year: i32,
    pub payment_date: NaiveDate,
    pub payment_method: String,
    pub house_bank: Option<String>,
    pub bank_account: Option<String>,
    pub currency: String,
    pub total_amount: Decimal,
    pub status: PaymentStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaymentStatus {
    Created,
    Executed,
    Reversed,
}
