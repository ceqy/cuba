use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Incident {
    pub incident_id: Uuid,
    pub incident_code: String,
    pub category: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub incident_datetime: Option<DateTime<Utc>>,
    pub reported_by: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
