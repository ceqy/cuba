use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TransferOrder {
    pub to_id: Uuid,
    pub to_number: String,
    pub warehouse_number: String,
    pub movement_type: String,
    pub reference_doc_type: Option<String>,
    pub reference_doc_number: Option<String>,
    pub status: String,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    #[sqlx(skip)]
    pub items: Vec<TransferOrderItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TransferOrderItem {
    pub item_id: Uuid,
    pub to_id: Uuid,
    pub item_number: i32,
    pub material: String,
    pub target_quantity: Decimal,
    pub actual_quantity: Decimal,
    pub unit: String,
    pub src_storage_type: Option<String>,
    pub src_storage_bin: Option<String>,
    pub dst_storage_type: Option<String>,
    pub dst_storage_bin: Option<String>,
    pub batch: Option<String>,
    pub confirmed: bool,
}
