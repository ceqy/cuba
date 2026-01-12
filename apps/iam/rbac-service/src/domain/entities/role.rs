use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parent_id: Option<String>,
    pub tenant_id: String,
    pub is_immutable: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
