//! AP Service Commands
//! Command objects for CQRS pattern

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Command to create or update a supplier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostSupplierCommand {
    pub supplier_id: String,
    pub business_partner_id: Option<String>,
    pub name: String,
    pub account_group: String,
    pub street: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub telephone: Option<String>,
    pub email: Option<String>,
    pub company_code: String,
    pub reconciliation_account: String,
    pub payment_terms: Option<String>,
    pub check_double_invoice: bool,
    pub purchasing_organization: Option<String>,
    pub order_currency: Option<String>,
}

/// Command to post an invoice (FB60/MIRO)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostInvoiceCommand {
    pub company_code: String,
    pub supplier_id: String,
    pub document_date: NaiveDate,
    pub posting_date: NaiveDate,
    pub currency: String,
    pub reference_document: Option<String>,
    pub header_text: Option<String>,
    pub items: Vec<InvoiceItemCommand>,
    pub ledger: Option<String>,   // 分类账
    pub ledger_type: Option<i32>, // 分类账类型
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceItemCommand {
    pub gl_account: String,
    pub debit_credit: String, // S or H
    pub amount: Decimal,
    pub cost_center: Option<String>,
    pub item_text: Option<String>,
    pub purchase_order: Option<String>,
    pub po_item_number: Option<i32>,
}

/// Query to list open items for a supplier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListOpenItemsQuery {
    pub business_partner_id: String,
    pub company_code: String,
    pub account_type: String, // K for vendor
    pub include_cleared: bool,
    pub page_size: i32,
    pub page_token: Option<String>,
}

/// Command to clear open items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearOpenItemsCommand {
    pub company_code: String,
    pub account_type: String,
    pub account_id: String,
    pub item_ids: Vec<OpenItemIdentifier>,
    pub reason_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenItemIdentifier {
    pub document_number: String,
    pub fiscal_year: i32,
    pub line_item_number: i32,
}
