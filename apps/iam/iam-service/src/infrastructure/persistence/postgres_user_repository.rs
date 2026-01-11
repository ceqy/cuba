use crate::domain::aggregates::user::{User, UserId};
use crate::domain::repositories::UserRepository;
use async_trait::async_trait;
use cuba_core::repository::Repository;
use cuba_database::DbPool;
use anyhow::{Result, Context};

pub struct PostgresUserRepository {
    pool: DbPool,
}

impl PostgresUserRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<User> for PostgresUserRepository {
    type Id = UserId;

    async fn find_by_id(&self, id: &Self::Id) -> Result<Option<User>> {
        let row: Option<UserRow> = sqlx::query_as(
            r#"
            SELECT id, username, email, password_hash, tenant_id, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id.clone().into_inner())
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find user by id")?;

        Ok(row.map(|r| r.into_domain()))
    }

    async fn save(&self, entity: &User) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, tenant_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE
            SET username = $2, email = $3, password_hash = $4, updated_at = $7
            "#,
        )
        .bind(entity.id.clone().into_inner())
        .bind(&entity.username)
        .bind(&entity.email)
        .bind(&entity.password_hash)
        .bind(&entity.tenant_id)
        .bind(entity.created_at)
        .bind(entity.updated_at)
        .execute(&self.pool)
        .await
        .context("Failed to save user")?;

        Ok(())
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let row: Option<UserRow> = sqlx::query_as(
            r#"
            SELECT id, username, email, password_hash, tenant_id, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find user by username")?;

        Ok(row.map(|r| r.into_domain()))
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let row: Option<UserRow> = sqlx::query_as(
            r#"
            SELECT id, username, email, password_hash, tenant_id, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find user by email")?;

        Ok(row.map(|r| r.into_domain()))
    }
}

// Internal row structure for mapping
#[derive(sqlx::FromRow)]
struct UserRow {
    id: uuid::Uuid,
    username: String,
    email: String,
    password_hash: String,
    tenant_id: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl UserRow {
    fn into_domain(self) -> User {
        User {
            id: UserId::from_uuid(self.id),
            username: self.username,
            email: self.email,
            password_hash: self.password_hash,
            tenant_id: self.tenant_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
