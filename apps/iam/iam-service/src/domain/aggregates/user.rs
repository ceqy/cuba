use cuba_core::domain::{AggregateRoot, Entity};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub tenant_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl AggregateRoot for User {}
impl Entity for User {
    type Id = UserId;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl User {
    pub fn new(username: String, email: String, password_hash: String, tenant_id: Option<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: UserId::new(),
            username,
            email,
            password_hash,
            tenant_id,
            created_at: now,
            updated_at: now,
        }
    }
}
