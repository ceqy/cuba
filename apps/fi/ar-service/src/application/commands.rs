use serde::Deserialize;
use rust_decimal::Decimal;
use chrono::NaiveDate;

#[derive(Debug, Deserialize)]
pub struct PostCustomerCommand {
    pub customer_id: String,
    pub business_partner_id: Option<String>,
    pub name: String,
    pub account_group: String,
    
    pub street: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    
    pub company_code: String,
    pub reconciliation_account: String,
    pub payment_terms: Option<String>,
    
    pub sales_organization: Option<String>,
    pub order_currency: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListOpenItemsQuery {
    pub customer_id: String,
    pub company_code: String,
    pub include_cleared: bool,
    pub page_size: i32,
    pub page_token: Option<String>,
}

/// Command to post a sales invoice (FB70 - Customer Invoice)
#[derive(Debug, Clone, Deserialize)]
pub struct PostSalesInvoiceCommand {
    pub company_code: String,
    pub customer_id: String,
    pub document_date: NaiveDate,
    pub posting_date: NaiveDate,
    pub currency: String,
    pub reference_document: Option<String>,
    pub header_text: Option<String>,
    pub items: Vec<SalesInvoiceItemCommand>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SalesInvoiceItemCommand {
    pub gl_account: String,
    pub debit_credit: String, // S or H
    pub amount: Decimal,
    pub cost_center: Option<String>,
    pub item_text: Option<String>,
}
