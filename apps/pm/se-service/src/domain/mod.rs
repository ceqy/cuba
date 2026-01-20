use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RFQ {
    pub rfq_id: Uuid,
    pub rfq_number: String,
    pub company_code: String,
    pub purchasing_org: Option<String>,
    pub quote_deadline: Option<NaiveDate>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    #[sqlx(skip)]
    pub items: Vec<RFQItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RFQItem {
    pub item_id: Uuid,
    pub rfq_id: Uuid,
    pub item_number: i32,
    pub material: String,
    pub description: Option<String>,
    pub quantity: Option<Decimal>,
    pub unit: String,
    pub delivery_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SupplierQuote {
    pub quote_id: Uuid,
    pub quote_number: String,
    pub rfq_id: Uuid,
    pub supplier_id: String,
    pub validity_end_date: Option<NaiveDate>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    #[sqlx(skip)]
    pub items: Vec<QuoteItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuoteItem {
    pub quote_item_id: Uuid,
    pub quote_id: Uuid,
    pub rfq_item_number: i32,
    pub quantity: Option<Decimal>,
    pub unit: String,
    pub net_price: Option<Decimal>,
    pub currency: String,
    pub notes: Option<String>,
}
