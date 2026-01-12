use chrono::{DateTime, Utc}; use serde::{Deserialize, Serialize}; use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SystemSetting { pub setting_id: Uuid, pub setting_key: String, pub setting_value: Option<String>, pub description: Option<String>, pub updated_at: DateTime<Utc> }
