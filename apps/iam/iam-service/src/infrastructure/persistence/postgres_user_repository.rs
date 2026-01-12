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
        let user_id = id.clone().into_inner();
        let row: Option<UserRow> = sqlx::query_as(
            r#"
            SELECT id, username, email, password_hash, tenant_id, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find user by id")?;

        if let Some(user_row) = row {
            let roles: Vec<String> = sqlx::query_scalar(
                "SELECT role_id FROM user_roles WHERE user_id = $1"
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch user roles")?;

            Ok(Some(user_row.into_domain(roles)))
        } else {
            Ok(None)
        }
    }

    async fn save(&self, entity: &User) -> Result<()> {
        let mut tx = self.pool.begin().await.context("Failed to start transaction")?;

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
        .execute(&mut *tx)
        .await
        .context("Failed to save user")?;

        // Sync roles
        sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
            .bind(entity.id.clone().into_inner())
            .execute(&mut *tx)
            .await
            .context("Failed to clear old roles")?;

        for role_id in &entity.roles {
            sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2)")
                .bind(entity.id.clone().into_inner())
                .bind(role_id)
                .execute(&mut *tx)
                .await
                .context("Failed to insert user role")?;
        }

        tx.commit().await.context("Failed to commit transaction")?;

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

        if let Some(user_row) = row {
            let roles: Vec<String> = sqlx::query_scalar(
                "SELECT role_id FROM user_roles WHERE user_id = $1"
            )
            .bind(user_row.id)
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch user roles")?;

            Ok(Some(user_row.into_domain(roles)))
        } else {
            Ok(None)
        }
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

        if let Some(user_row) = row {
            let roles: Vec<String> = sqlx::query_scalar(
                "SELECT role_id FROM user_roles WHERE user_id = $1"
            )
            .bind(user_row.id)
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch user roles")?;

            Ok(Some(user_row.into_domain(roles)))
        } else {
            Ok(None)
        }
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
    fn into_domain(self, roles: Vec<String>) -> User {
        User {
            id: UserId::from_uuid(self.id),
            username: self.username,
            email: self.email,
            password_hash: self.password_hash,
            tenant_id: self.tenant_id,
            roles,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
