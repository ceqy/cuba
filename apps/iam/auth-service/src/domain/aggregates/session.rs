use chrono::{DateTime, Utc};
use cuba_core::domain::{AggregateRoot, Entity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserSession {
    pub id: String,
    pub user_id: String,
    pub tenant_id: String,
    pub refresh_token: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub is_revoked: bool,
}

impl Entity for UserSession {
    type Id = String;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl AggregateRoot for UserSession {}

impl UserSession {
    pub fn new(
        user_id: String,
        tenant_id: String,
        refresh_token: String,
        user_agent: Option<String>,
        ip_address: Option<String>,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            tenant_id,
            refresh_token,
            user_agent,
            ip_address,
            expires_at,
            created_at: Utc::now(),
            last_seen_at: Utc::now(),
            is_revoked: false,
        }
    }
}
