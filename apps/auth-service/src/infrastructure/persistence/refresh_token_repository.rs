//! PostgreSQL Refresh Token Repository implementation

use crate::domain::repositories::{RefreshTokenData, RefreshTokenRepository, RepositoryError};
use crate::domain::value_objects::UserId;
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

pub struct PgRefreshTokenRepository {
    pool: Arc<PgPool>,
}

impl PgRefreshTokenRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RefreshTokenRepository for PgRefreshTokenRepository {
    async fn save(&self, token: &RefreshTokenData) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (id, user_id, token, expires_at, created_at, revoked_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE SET
                revoked_at = EXCLUDED.revoked_at
            "#,
        )
        .bind(Uuid::parse_str(&token.id).unwrap_or_else(|_| Uuid::new_v4()))
        .bind(token.user_id.as_uuid())
        .bind(&token.token_hash)
        .bind(token.expires_at)
        .bind(token.created_at)
        .bind(if token.is_revoked { Some(chrono::Utc::now()) } else { None })
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_hash(&self, token_hash: &str) -> Result<Option<RefreshTokenData>, RepositoryError> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, token, expires_at, created_at, revoked_at
            FROM refresh_tokens
            WHERE token = $1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                let id: Uuid = row.get("id");
                let user_id: Uuid = row.get("user_id");
                let token_hash: String = row.get("token");
                let expires_at: chrono::DateTime<chrono::Utc> = row.get("expires_at");
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                let revoked_at: Option<chrono::DateTime<chrono::Utc>> = row.get("revoked_at");

                Ok(Some(RefreshTokenData {
                    id: id.to_string(),
                    user_id: UserId::from(user_id),
                    token_hash,
                    expires_at,
                    is_revoked: revoked_at.is_some(),
                    created_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn revoke(&self, token_hash: &str) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE refresh_tokens SET revoked_at = NOW() WHERE token = $1 AND revoked_at IS NULL")
            .bind(token_hash)
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn revoke_all_for_user(&self, user_id: &UserId) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE refresh_tokens SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL")
            .bind(user_id.as_uuid())
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn cleanup_expired(&self) -> Result<u64, RepositoryError> {
        let result = sqlx::query("DELETE FROM refresh_tokens WHERE expires_at < NOW() OR revoked_at < NOW() - INTERVAL '30 days'")
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected())
    }
}
