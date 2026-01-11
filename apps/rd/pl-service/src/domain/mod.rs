use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillOfMaterial {
    pub bom_id: Uuid,
    pub material: String,
    pub plant: String,
    pub bom_usage: String,
    pub bom_status: String,
    pub base_quantity: Decimal,
    pub alternative_bom: String,
    pub valid_from: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub items: Vec<BOMItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BOMItem {
    pub item_id: Uuid,
    pub bom_id: Uuid,
    pub item_node: String,
    pub item_category: String,
    pub component_material: String,
    pub component_quantity: Decimal,
    pub component_unit: String,
    pub item_text: Option<String>,
    pub recursive_allowed: bool,
}
