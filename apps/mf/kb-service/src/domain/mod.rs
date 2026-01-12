use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ControlCycle {
    pub cycle_id: Uuid,
    pub cycle_number: String,
    pub material: String,
    pub plant: String,
    pub supply_area: Option<String>,
    pub number_of_kanbans: i32,
    pub qty_per_kanban: Decimal,
    pub unit: String,
    pub replenishment_strategy: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct KanbanContainer {
    pub container_id: Uuid,
    pub container_code: String,
    pub cycle_id: Uuid,
    pub status: String,
    pub last_status_change: DateTime<Utc>,
}
