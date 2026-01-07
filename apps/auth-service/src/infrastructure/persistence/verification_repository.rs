//! PostgreSQL Verification Token Repository implementation

use crate::domain::repositories::{VerificationRepository, VerificationTokenData, VerificationTokenType, RepositoryError};
use crate::domain::value_objects::UserId;
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

pub struct PgVerificationRepository {
    pool: Arc<PgPool>,
}

impl PgVerificationRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VerificationRepository for PgVerificationRepository {
    async fn save(&self, token: &VerificationTokenData) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO verification_tokens (id, user_id, token_type, token_hash, expires_at, created_at, used_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                used_at = EXCLUDED.used_at
            "#,
        )
        .bind(Uuid::parse_str(&token.id).unwrap_or_else(|_| Uuid::new_v4()))
        .bind(token.user_id.as_uuid())
        .bind(token.token_type.to_string())
        .bind(&token.token_hash)
        .bind(token.expires_at)
        .bind(token.created_at)
        .bind(token.used_at)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_hash(&self, token_hash: &str, token_type: VerificationTokenType) -> Result<Option<VerificationTokenData>, RepositoryError> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, token_type, token_hash, expires_at, created_at, used_at
            FROM verification_tokens
            WHERE token_hash = $1 AND token_type = $2
            "#,
        )
        .bind(token_hash)
        .bind(token_type.to_string())
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                let id: Uuid = row.get("id");
                let user_id: Uuid = row.get("user_id");
                let token_type_str: String = row.get("token_type");
                let token_hash: String = row.get("token_hash");
                let expires_at: chrono::DateTime<chrono::Utc> = row.get("expires_at");
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                let used_at: Option<chrono::DateTime<chrono::Utc>> = row.get("used_at");

                Ok(Some(VerificationTokenData {
                    id: id.to_string(),
                    user_id: UserId::from(user_id),
                    token_type: VerificationTokenType::from(token_type_str),
                    token_hash,
                    expires_at,
                    created_at,
                    used_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn mark_as_used(&self, token_hash: &str) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE verification_tokens SET used_at = NOW() WHERE token_hash = $1 AND used_at IS NULL")
            .bind(token_hash)
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_expired(&self, expires_before: chrono::DateTime<chrono::Utc>) -> Result<u64, RepositoryError> {
        let result = sqlx::query("DELETE FROM verification_tokens WHERE expires_at < $1")
            .bind(expires_before)
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected())
    }
}
