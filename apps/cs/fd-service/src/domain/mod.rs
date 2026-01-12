use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ServiceOrder {
    pub order_id: Uuid,
    pub order_number: String,
    pub order_type: String,
    pub customer_id: String,
    pub description: Option<String>,
    pub planned_start: Option<DateTime<Utc>>,
    pub assigned_technician_id: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
