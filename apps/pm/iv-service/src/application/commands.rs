use serde::Deserialize;
use rust_decimal::Decimal;
use chrono::NaiveDate;

#[derive(Debug, Deserialize)]
pub struct ReceiveInvoiceCommand {
    pub company_code: String,
    pub supplier_invoice_number: String,
    pub document_date: NaiveDate,
    pub gross_amount: Decimal,
    pub tax_amount: Decimal,
    pub items: Vec<InvoiceItemCmd>,
}

#[derive(Debug, Deserialize)]
pub struct InvoiceItemCmd {
    pub item_number: i32,
    pub po_number: Option<String>,
    pub po_item: Option<i32>,
    pub material: Option<String>,
    pub quantity: Decimal,
    pub amount: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct MatchInvoiceCommand {
    pub invoice_id: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
pub struct PostInvoiceCommand {
    pub invoice_id: uuid::Uuid,
    pub accept_variances: bool,
}
