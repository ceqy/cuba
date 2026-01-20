use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
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
    pub baseline_date: Option<NaiveDate>,

    pub customer_id: String,
    pub currency: String,
    pub total_amount: Decimal,

    pub reference: Option<String>,
    // ACDOCA minimal alignment
    pub ledger: Option<String>,
    pub special_gl_indicator: Option<String>,
    pub payment_terms: Option<String>,
    pub payment_method: Option<String>,
    pub payment_block: Option<String>,
    pub transaction_type: Option<String>,
    pub reference_transaction_type: Option<String>,
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
    pub baseline_date: Option<NaiveDate>,

    pub currency: String,
    pub original_amount: Decimal,
    pub open_amount: Decimal,

    pub is_cleared: bool,
    pub payment_block: Option<String>,
    pub reference_document: Option<String>,
    pub item_text: Option<String>,

    // ACDOCA minimal alignment
    pub ledger: Option<String>,
    pub special_gl_indicator: Option<String>,
    pub payment_method: Option<String>,
    pub payment_terms: Option<String>,
    pub dunning_block: Option<String>,
    pub dunning_level: Option<i32>,
    pub transaction_type: Option<String>,
    pub reference_transaction_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invoice_supports_acdoca_fields() {
        let invoice = Invoice {
            invoice_id: Uuid::new_v4(),
            document_number: Some("DR-TEST".to_string()),
            company_code: "1000".to_string(),
            fiscal_year: 2026,
            document_date: NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
            posting_date: NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
            baseline_date: Some(NaiveDate::from_ymd_opt(2026, 1, 19).unwrap()),
            customer_id: "CUST001".to_string(),
            currency: "CNY".to_string(),
            total_amount: Decimal::new(12000, 2),
            reference: None,
            ledger: Some("0L".to_string()),
            special_gl_indicator: None,
            payment_terms: Some("0002".to_string()),
            payment_method: Some("T".to_string()),
            payment_block: None,
            transaction_type: Some("AR".to_string()),
            reference_transaction_type: Some("ARIN".to_string()),
            status: InvoiceStatus::Posted,
            items: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(invoice.ledger.as_deref(), Some("0L"));
        assert_eq!(invoice.payment_terms.as_deref(), Some("0002"));
    }

    #[test]
    fn open_item_supports_acdoca_fields() {
        let item = OpenItem {
            open_item_id: Uuid::new_v4(),
            document_number: "DR-TEST".to_string(),
            fiscal_year: 2026,
            company_code: "1000".to_string(),
            line_item_number: 1,
            customer_id: "CUST001".to_string(),
            doc_type: "DR".to_string(),
            posting_date: NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2026, 2, 18).unwrap(),
            baseline_date: Some(NaiveDate::from_ymd_opt(2026, 1, 19).unwrap()),
            currency: "CNY".to_string(),
            original_amount: Decimal::new(12000, 2),
            open_amount: Decimal::new(12000, 2),
            is_cleared: false,
            payment_block: None,
            reference_document: None,
            item_text: None,
            ledger: Some("0L".to_string()),
            special_gl_indicator: None,
            payment_method: Some("T".to_string()),
            payment_terms: Some("0002".to_string()),
            dunning_block: None,
            dunning_level: Some(1),
            transaction_type: Some("AR".to_string()),
            reference_transaction_type: Some("ARIN".to_string()),
        };

        assert_eq!(item.payment_terms.as_deref(), Some("0002"));
        assert_eq!(item.dunning_level, Some(1));
    }
}
