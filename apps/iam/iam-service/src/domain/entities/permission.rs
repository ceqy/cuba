use cuba_core::domain::{Entity, AggregateRoot};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: String,
    pub code: String,
    pub resource: String,
    pub action: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl AggregateRoot for Permission {}
impl Entity for Permission {
    type Id = String;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}
