use rust_decimal::Decimal;
use chrono::{NaiveDate, DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SalesOrder {
    pub order_id: Uuid,
    pub order_number: String,
    
    pub order_type: String,
    pub sales_org: String,
    pub distribution_channel: String,
    pub division: String,
    
    pub sold_to_party: String,
    pub ship_to_party: Option<String>,
    
    pub customer_po: Option<String>,
    pub customer_po_date: Option<NaiveDate>,
    
    pub document_date: NaiveDate,
    pub requested_delivery_date: Option<NaiveDate>,
    
    pub currency: String,
    pub net_value: Decimal,
    
    pub pricing_procedure: Option<String>,
    pub shipping_conditions: Option<String>,
    
    pub overall_status: String,
    pub delivery_block: Option<String>,
    pub billing_block: Option<String>,
    
    #[sqlx(skip)]
    pub items: Vec<SalesOrderItem>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SalesOrderItem {
    pub item_id: Uuid,
    pub order_id: Uuid,
    pub item_number: i32,
    
    pub material: String,
    pub item_description: Option<String>,
    
    pub order_quantity: Decimal,
    pub sales_unit: String,
    
    pub plant: Option<String>,
    pub storage_location: Option<String>,
    
    pub net_value: Decimal,
    pub tax_amount: Option<Decimal>,
    
    pub item_category: Option<String>,
    pub rejection_reason: Option<String>,
    
    pub higher_level_item: Option<i32>,

    #[sqlx(skip)]
    pub schedule_lines: Vec<SalesOrderScheduleLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SalesOrderScheduleLine {
    pub schedule_line_id: Uuid,
    pub item_id: Uuid,
    pub schedule_line_number: i32,
    pub delivery_date: NaiveDate,
    pub confirmed_quantity: Decimal,
}
