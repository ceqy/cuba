use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PurchaseOrder {
    pub order_id: Uuid,
    pub order_number: String,

    pub document_type: i32,
    pub company_code: String,
    pub purchasing_org: String,
    pub purchasing_group: String,

    pub supplier: String,
    pub order_date: NaiveDate,

    pub currency: String,
    pub payment_terms: Option<String>,
    pub incoterms: Option<String>,
    pub incoterms_location: Option<String>,

    pub complete_delivery: bool,
    pub release_status: i32,

    #[sqlx(skip)]
    pub items: Vec<PurchaseOrderItem>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PurchaseOrderItem {
    pub item_id: Uuid,
    pub order_id: Uuid,

    pub item_number: i32,
    pub item_category: i32,

    pub material: String,
    pub short_text: Option<String>,

    pub plant: String,
    pub storage_location: Option<String>,
    pub material_group: Option<String>,

    pub quantity: Decimal,
    pub quantity_unit: String,

    pub net_price: Decimal,
    pub price_unit: i32,
    pub currency: String,

    pub gr_based_iv: bool,
    pub tax_code: Option<String>,
    pub account_assignment_category: Option<String>,

    pub requisition_number: Option<String>,
    pub requisition_item: Option<i32>,
    pub deletion_indicator: bool,

    #[sqlx(skip)]
    pub schedule_lines: Vec<PurchaseOrderScheduleLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PurchaseOrderScheduleLine {
    pub schedule_line_id: Uuid,
    pub item_id: Uuid,
    pub schedule_line_number: i32,
    pub delivery_date: NaiveDate,
    pub scheduled_quantity: Decimal,
    pub goods_receipt_quantity: Decimal,
}
