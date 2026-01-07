//! PostgreSQL API Key Repository implementation

use crate::domain::repositories::{ApiKeyRepository, ApiKeyData, RepositoryError};
use crate::domain::value_objects::UserId;
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

pub struct PgApiKeyRepository {
    pool: Arc<PgPool>,
}

impl PgApiKeyRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ApiKeyRepository for PgApiKeyRepository {
    async fn save(&self, key: &ApiKeyData) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO api_keys (id, name, prefix, key_hash, scopes, user_id, tenant_id, expires_at, created_at, revoked_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO UPDATE SET
                revoked_at = EXCLUDED.revoked_at
            "#,
        )
        .bind(Uuid::parse_str(&key.id).unwrap_or_else(|_| Uuid::new_v4()))
        .bind(&key.name)
        .bind(&key.prefix)
        .bind(&key.key_hash)
        .bind(&key.scopes)
        .bind(key.user_id.as_uuid())
        .bind(&key.tenant_id)
        .bind(key.expires_at)
        .bind(key.created_at)
        .bind(key.revoked_at)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<ApiKeyData>, RepositoryError> {
        let uuid = Uuid::parse_str(id).map_err(|_| RepositoryError::DatabaseError("Invalid UUID".to_string()))?;
        
        let row = sqlx::query(
            "SELECT id, name, prefix, key_hash, scopes, user_id, tenant_id, expires_at, created_at, revoked_at FROM api_keys WHERE id = $1"
        )
        .bind(uuid)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                let user_id: Uuid = row.get("user_id");
                Ok(Some(ApiKeyData {
                    id: row.get::<Uuid, _>("id").to_string(),
                    name: row.get("name"),
                    prefix: row.get("prefix"),
                    key_hash: row.get("key_hash"),
                    scopes: row.get::<Vec<String>, _>("scopes"),
                    user_id: UserId::from(user_id),
                    tenant_id: row.get::<Option<String>, _>("tenant_id").unwrap_or_default(),
                    expires_at: row.get("expires_at"),
                    created_at: row.get("created_at"),
                    revoked_at: row.get("revoked_at"),
                }))
            }
            None => Ok(None),
        }
    }

    async fn find_by_user(&self, user_id: &UserId, limit: i64, offset: i64) -> Result<Vec<ApiKeyData>, RepositoryError> {
        let rows = sqlx::query(
            "SELECT id, name, prefix, key_hash, scopes, user_id, tenant_id, expires_at, created_at, revoked_at FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(user_id.as_uuid())
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let mut keys = Vec::new();
        for row in rows {
            let user_id: Uuid = row.get("user_id");
            keys.push(ApiKeyData {
                id: row.get::<Uuid, _>("id").to_string(),
                name: row.get("name"),
                prefix: row.get("prefix"),
                key_hash: row.get("key_hash"),
                scopes: row.get::<Vec<String>, _>("scopes"),
                user_id: UserId::from(user_id),
                tenant_id: row.get::<Option<String>, _>("tenant_id").unwrap_or_default(),
                expires_at: row.get("expires_at"),
                created_at: row.get("created_at"),
                revoked_at: row.get("revoked_at"),
            });
        }
        Ok(keys)
    }

    async fn count_by_user(&self, user_id: &UserId) -> Result<i64, RepositoryError> {
        let row = sqlx::query("SELECT COUNT(*) FROM api_keys WHERE user_id = $1")
            .bind(user_id.as_uuid())
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(row.get(0))
    }

    async fn revoke(&self, id: &str) -> Result<(), RepositoryError> {
        let uuid = Uuid::parse_str(id).map_err(|_| RepositoryError::DatabaseError("Invalid UUID".to_string()))?;
        
        sqlx::query("UPDATE api_keys SET revoked_at = NOW() WHERE id = $1")
            .bind(uuid)
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
