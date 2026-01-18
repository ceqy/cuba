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

    // ACDOCA minimal alignment
    pub ledger: Option<String>,
    pub special_gl_indicator: Option<String>,
    pub payment_terms: Option<String>,
    pub payment_method: Option<String>,
    pub payment_block: Option<String>,
    pub transaction_type: Option<String>,
    pub reference_transaction_type: Option<String>,
    
    // Status
    pub status: String, // InvoiceStatus
    pub clearing_document: Option<String>,
    pub clearing_date: Option<NaiveDate>,
    
    // Line items
    #[sqlx(skip)]
    pub items: Vec<InvoiceItem>,
    
    // Audit
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
// ...
/// Invoice line item
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct InvoiceItem {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub line_item_number: i32,
    pub gl_account: String,
    pub debit_credit_indicator: String, // DebitCredit (S/H)
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
    // ACDOCA minimal alignment
    pub ledger: Option<String>,
    pub special_gl_indicator: Option<String>,
    pub payment_method: Option<String>,
    pub payment_terms: Option<String>,
    pub dunning_block: Option<String>,
    pub dunning_level: Option<i32>,
    pub transaction_type: Option<String>,
    pub reference_transaction_type: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invoice_supports_acdoca_fields() {
        let invoice = Invoice {
            id: Uuid::new_v4(),
            document_number: "INV-TEST".to_string(),
            company_code: "1000".to_string(),
            fiscal_year: 2026,
            document_type: "KR".to_string(),
            supplier_id: Uuid::new_v4(),
            document_date: NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
            posting_date: NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2026, 2, 18).unwrap(),
            baseline_date: Some(NaiveDate::from_ymd_opt(2026, 1, 19).unwrap()),
            currency: "CNY".to_string(),
            total_amount: Decimal::new(10000, 2),
            tax_amount: Decimal::ZERO,
            reference_document: None,
            header_text: None,
            ledger: Some("0L".to_string()),
            special_gl_indicator: Some("".to_string()),
            payment_terms: Some("0001".to_string()),
            payment_method: Some("T".to_string()),
            payment_block: None,
            transaction_type: Some("AP".to_string()),
            reference_transaction_type: Some("APIN".to_string()),
            status: "OPEN".to_string(),
            clearing_document: None,
            clearing_date: None,
            items: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(invoice.ledger.as_deref(), Some("0L"));
        assert_eq!(invoice.payment_terms.as_deref(), Some("0001"));
        assert_eq!(invoice.transaction_type.as_deref(), Some("AP"));
    }

    #[test]
    fn open_item_supports_acdoca_fields() {
        let item = OpenItem {
            id: Uuid::new_v4(),
            document_number: "INV-TEST".to_string(),
            company_code: "1000".to_string(),
            fiscal_year: 2026,
            line_item_number: 1,
            supplier_id: None,
            account_type: "K".to_string(),
            posting_date: NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2026, 2, 18).unwrap(),
            baseline_date: Some(NaiveDate::from_ymd_opt(2026, 1, 19).unwrap()),
            currency: "CNY".to_string(),
            original_amount: Decimal::new(10000, 2),
            open_amount: Decimal::new(10000, 2),
            is_cleared: false,
            clearing_document: None,
            clearing_date: None,
            reference_document: None,
            item_text: None,
            payment_block: Some("A".to_string()),
            ledger: Some("0L".to_string()),
            special_gl_indicator: None,
            payment_method: Some("T".to_string()),
            payment_terms: Some("0001".to_string()),
            dunning_block: None,
            dunning_level: Some(1),
            transaction_type: Some("AP".to_string()),
            reference_transaction_type: Some("APIN".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(item.payment_method.as_deref(), Some("T"));
        assert_eq!(item.dunning_level, Some(1));
    }
}
