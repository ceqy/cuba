use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Batch {
    pub batch_id: Uuid,
    pub batch_number: String,
    pub material: String,
    pub plant: String,
    pub production_date: Option<NaiveDate>,
    pub expiration_date: Option<NaiveDate>,
    pub supplier_batch: Option<String>,
    pub origin_batch: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchHistoryEvent {
    pub event_id: Uuid,
    pub batch_id: Uuid,
    pub event_time: DateTime<Utc>,
    pub event_type: String,
    pub user_id: Option<String>,
    pub details: Option<String>,
    pub document_number: Option<String>,
    pub document_type: Option<String>,
}
