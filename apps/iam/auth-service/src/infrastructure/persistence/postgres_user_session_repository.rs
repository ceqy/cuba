use crate::domain::{UserSession, UserSessionRepository};
use async_trait::async_trait;
use cuba_core::repository::Repository;
use sqlx::{Pool, Postgres};

pub struct PostgresUserSessionRepository {
    pool: Pool<Postgres>,
}

impl PostgresUserSessionRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<UserSession> for PostgresUserSessionRepository {
    type Id = String;

    async fn find_by_id(&self, id: &Self::Id) -> anyhow::Result<Option<UserSession>> {
        let row = sqlx::query_as::<_, UserSession>(
            r#"
            SELECT id, user_id, tenant_id, refresh_token, 
                   user_agent, ip_address, expires_at, created_at, last_seen_at, is_revoked
            FROM user_sessions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn save(&self, entity: &UserSession) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO user_sessions (id, user_id, tenant_id, refresh_token, user_agent, ip_address, expires_at, created_at, last_seen_at, is_revoked)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO UPDATE SET
                refresh_token = EXCLUDED.refresh_token,
                last_seen_at = EXCLUDED.last_seen_at,
                is_revoked = EXCLUDED.is_revoked
            "#,
        )
        .bind(&entity.id)
        .bind(&entity.user_id)
        .bind(&entity.tenant_id)
        .bind(&entity.refresh_token)
        .bind(&entity.user_agent)
        .bind(&entity.ip_address)
        .bind(entity.expires_at)
        .bind(entity.created_at)
        .bind(entity.last_seen_at)
        .bind(entity.is_revoked)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[async_trait]
impl UserSessionRepository for PostgresUserSessionRepository {
    async fn find_by_refresh_token(
        &self,
        token: &str,
    ) -> Result<Option<UserSession>, anyhow::Error> {
        let row = sqlx::query_as::<_, UserSession>(
            r#"
            SELECT id, user_id, tenant_id, refresh_token, 
                   user_agent, ip_address, expires_at, created_at, last_seen_at, is_revoked
            FROM user_sessions
            WHERE refresh_token = $1 AND is_revoked = FALSE
            "#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn revoke_by_user_id(&self, user_id: &str) -> Result<(), anyhow::Error> {
        sqlx::query("UPDATE user_sessions SET is_revoked = TRUE WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
