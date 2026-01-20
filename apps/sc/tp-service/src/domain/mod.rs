use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Shipment {
    pub shipment_id: Uuid,
    pub shipment_number: String,
    pub shipment_type: Option<String>,
    pub transportation_planning_point: Option<String>,
    pub carrier: Option<String>,
    pub overall_status: String,
    pub planned_departure: Option<DateTime<Utc>>,
    pub planned_arrival: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    #[sqlx(skip)]
    pub items: Vec<ShipmentItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ShipmentItem {
    pub item_id: Uuid,
    pub shipment_id: Uuid,
    pub item_number: i32,
    pub delivery_number: String,
    pub total_weight: Option<Decimal>,
    pub weight_unit: String,
    pub volume: Option<Decimal>,
    pub volume_unit: String,
}
