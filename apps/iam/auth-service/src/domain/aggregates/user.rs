use cuba_core::domain::{AggregateRoot, Entity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub tenant_id: String,
    pub roles: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl AggregateRoot for User {}
impl Entity for User {
    type Id = String;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl User {
    pub fn new(username: String, email: String, password_hash: String, tenant_id: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            username,
            email,
            password_hash,
            tenant_id,
            roles: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}
