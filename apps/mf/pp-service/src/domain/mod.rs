use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PlannedOrder {
    pub planned_order_id: Uuid,
    pub planned_order_number: String,

    pub material: String,
    pub plant: String,
    pub planning_plant: String,

    pub order_quantity: Decimal,
    pub quantity_unit: String,

    pub order_start_date: NaiveDate,
    pub order_finish_date: NaiveDate,

    pub mrp_controller: Option<String>,
    pub conversion_indicator: String,

    pub status: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
