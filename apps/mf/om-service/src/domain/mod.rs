use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubcontractingOrder {
    pub order_id: Uuid,
    pub purchase_order_number: String,
    pub supplier: String,
    pub company_code: String,
    pub purchasing_org: Option<String>,
    pub purchasing_group: Option<String>,
    pub created_at: DateTime<Utc>,
    pub items: Vec<SubcontractingItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubcontractingItem {
    pub item_id: Uuid,
    pub order_id: Uuid,
    pub item_number: i32,
    pub finished_good_material: String,
    pub order_quantity: Option<Decimal>,
    pub received_quantity: Decimal,
    pub unit: String,
    pub plant: Option<String>,
    pub components: Vec<SubcontractingComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubcontractingComponent {
    pub component_id: Uuid,
    pub item_id: Uuid,
    pub component_material: String,
    pub required_quantity: Option<Decimal>,
    pub issued_quantity: Decimal,
    pub unit: String,
}
