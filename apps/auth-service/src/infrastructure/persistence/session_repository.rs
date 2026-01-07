//! PostgreSQL Session Repository implementation

use crate::domain::repositories::{SessionRepository, SessionData, RepositoryError};
use crate::domain::value_objects::UserId;
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

pub struct PgSessionRepository {
    pool: Arc<PgPool>,
}

impl PgSessionRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepository for PgSessionRepository {
    async fn save(&self, session: &SessionData) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO user_sessions (session_id, user_id, device_name, ip_address, location, last_active, created_at, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (session_id) DO UPDATE SET
                device_name = EXCLUDED.device_name,
                ip_address = EXCLUDED.ip_address,
                location = EXCLUDED.location,
                last_active = EXCLUDED.last_active,
                expires_at = EXCLUDED.expires_at
            "#,
        )
        .bind(&session.session_id)
        .bind(session.user_id.as_uuid())
        .bind(&session.device_name)
        .bind(&session.ip_address)
        .bind(&session.location)
        .bind(session.last_active)
        .bind(session.created_at)
        .bind(session.expires_at)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, session_id: &str) -> Result<Option<SessionData>, RepositoryError> {
        let row = sqlx::query(
            "SELECT session_id, user_id, device_name, ip_address, location, last_active, created_at, expires_at FROM user_sessions WHERE session_id = $1",
        )
        .bind(session_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                let user_uuid: Uuid = row.get("user_id");
                Ok(Some(SessionData {
                    session_id: row.get("session_id"),
                    user_id: UserId::from(user_uuid),
                    device_name: row.get("device_name"),
                    ip_address: row.get("ip_address"),
                    location: row.get("location"),
                    last_active: row.get("last_active"),
                    created_at: row.get("created_at"),
                    expires_at: row.get("expires_at"),
                }))
            }
            None => Ok(None),
        }
    }

    async fn find_by_user(&self, user_id: &UserId) -> Result<Vec<SessionData>, RepositoryError> {
        let rows = sqlx::query(
            "SELECT session_id, user_id, device_name, ip_address, location, last_active, created_at, expires_at FROM user_sessions WHERE user_id = $1 ORDER BY last_active DESC",
        )
        .bind(user_id.as_uuid())
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let mut sessions = Vec::new();
        for row in rows {
            let user_uuid: Uuid = row.get("user_id");
            sessions.push(SessionData {
                session_id: row.get("session_id"),
                user_id: UserId::from(user_uuid),
                device_name: row.get("device_name"),
                ip_address: row.get("ip_address"),
                location: row.get("location"),
                last_active: row.get("last_active"),
                created_at: row.get("created_at"),
                expires_at: row.get("expires_at"),
            });
        }
        Ok(sessions)
    }

    async fn delete(&self, session_id: &str) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM user_sessions WHERE session_id = $1")
            .bind(session_id)
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_all_for_user(&self, user_id: &UserId) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
            .bind(user_id.as_uuid())
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_last_active(&self, session_id: &str, last_active: chrono::DateTime<chrono::Utc>) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE user_sessions SET last_active = $1 WHERE session_id = $2")
            .bind(last_active)
            .bind(session_id)
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn count_active(&self) -> Result<i64, RepositoryError> {
        // Count sessions that are not expired
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM user_sessions WHERE expires_at > NOW()")
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        
        Ok(row.0)
    }
}
