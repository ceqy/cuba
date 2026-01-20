use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Invoice {
    pub invoice_id: Uuid,
    pub company_code: String,
    pub supplier_invoice_number: String,
    pub document_date: NaiveDate,
    pub posting_date: Option<NaiveDate>,
    pub gross_amount: Decimal,
    pub tax_amount: Decimal,
    pub currency: String,
    pub payment_terms: Option<String>,
    pub header_text: Option<String>,
    pub status: String,
    pub document_number: Option<String>,
    pub created_at: DateTime<Utc>,
    #[sqlx(skip)]
    pub items: Vec<InvoiceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct InvoiceItem {
    pub item_id: Uuid,
    pub invoice_id: Uuid,
    pub item_number: i32,
    pub po_number: Option<String>,
    pub po_item: Option<i32>,
    pub material: Option<String>,
    pub short_text: Option<String>,
    pub quantity: Decimal,
    pub unit: String,
    pub amount: Decimal,
    pub tax_code: Option<String>,
}
