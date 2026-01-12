use cuba_core::domain::{Entity, AggregateRoot};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_role_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl AggregateRoot for Role {}
impl Entity for Role {
    type Id = String;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}
