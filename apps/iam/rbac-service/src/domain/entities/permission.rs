use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: String,
    pub code: String,
    pub resource: String,
    pub action: String,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
