use crate::domain::User;
use crate::domain::repositories::UserRepository;
use async_trait::async_trait;
use cuba_core::repository::Repository;
use cuba_database::DbPool;
use sqlx::Row;

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
    type Id = String;

    async fn save(&self, entity: &User) -> Result<(), anyhow::Error> {
        sqlx::query(
            "INSERT INTO users (id, username, email, password_hash, tenant_id, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (id) DO UPDATE SET 
             username = EXCLUDED.username, email = EXCLUDED.email, 
             password_hash = EXCLUDED.password_hash, updated_at = EXCLUDED.updated_at"
        )
        .bind(&entity.id)
        .bind(&entity.username)
        .bind(&entity.email)
        .bind(&entity.password_hash)
        .bind(&entity.tenant_id)
        .bind(entity.created_at)
        .bind(entity.updated_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    async fn find_by_id(&self, id: &String) -> Result<Option<User>, anyhow::Error> {
        let row = sqlx::query("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
            
        if let Some(row) = row {
             // Fetch roles from user_roles
             let roles: Vec<String> = sqlx::query_scalar(
                 "SELECT r.name FROM roles r JOIN user_roles ur ON r.id = ur.role_id WHERE ur.user_id = $1"
             )
             .bind(id)
             .fetch_all(&self.pool)
             .await
             .unwrap_or_default();

             Ok(Some(User {
                 id: row.try_get("id")?,
                 username: row.try_get("username")?,
                 email: row.try_get("email")?,
                 password_hash: row.try_get("password_hash")?,
                 tenant_id: row.try_get("tenant_id")?,
                 roles, 
                 created_at: row.try_get("created_at")?,
                 updated_at: row.try_get("updated_at")?,
             }))
        } else {
            Ok(None)
        }
    }
    
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, anyhow::Error> {
        let row = sqlx::query("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;
            
        if let Some(row) = row {
             let id: String = row.try_get("id")?;
             let roles: Vec<String> = sqlx::query_scalar(
                 "SELECT r.name FROM roles r JOIN user_roles ur ON r.id = ur.role_id WHERE ur.user_id = $1"
             )
             .bind(&id)
             .fetch_all(&self.pool)
             .await
             .unwrap_or_default();

             Ok(Some(User {
                 id,
                 username: row.try_get("username")?,
                 email: row.try_get("email")?,
                 password_hash: row.try_get("password_hash")?,
                 tenant_id: row.try_get("tenant_id")?,
                 roles, 
                 created_at: row.try_get("created_at")?,
                 updated_at: row.try_get("updated_at")?,
             }))
        } else {
            Ok(None)
        }
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, anyhow::Error> {
         let row = sqlx::query("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;
            
        if let Some(row) = row {
             let id: String = row.try_get("id")?;
             let roles: Vec<String> = sqlx::query_scalar(
                 "SELECT r.name FROM roles r JOIN user_roles ur ON r.id = ur.role_id WHERE ur.user_id = $1"
             )
             .bind(&id)
             .fetch_all(&self.pool)
             .await
             .unwrap_or_default();

             Ok(Some(User {
                 id,
                 username: row.try_get("username")?,
                 email: row.try_get("email")?,
                 password_hash: row.try_get("password_hash")?,
                 tenant_id: row.try_get("tenant_id")?,
                 roles, 
                 created_at: row.try_get("created_at")?,
                 updated_at: row.try_get("updated_at")?,
             }))
        } else {
            Ok(None)
        }
    }

    async fn list_users(&self, offset: i64, limit: i64) -> Result<(Vec<User>, i64), anyhow::Error> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;

        let rows = sqlx::query("SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let mut users = Vec::new();
        for row in rows {
             let id: String = row.try_get("id")?;
             let roles: Vec<String> = sqlx::query_scalar(
                 "SELECT r.name FROM roles r JOIN user_roles ur ON r.id = ur.role_id WHERE ur.user_id = $1"
             )
             .bind(&id)
             .fetch_all(&self.pool)
             .await
             .unwrap_or_default();

             users.push(User {
                 id,
                 username: row.try_get("username")?,
                 email: row.try_get("email")?,
                 password_hash: row.try_get("password_hash")?,
                 tenant_id: row.try_get("tenant_id")?,
                 roles, 
                 created_at: row.try_get("created_at")?,
                 updated_at: row.try_get("updated_at")?,
             });
        }
        
        Ok((users, count))
    }

    async fn delete(&self, user_id: &str) -> Result<(), anyhow::Error> {
        let mut tx = self.pool.begin().await?;
        
        // Delete user roles first (fkey constraint, though usually cascade)
        sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
            
        // Delete user
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
            
        tx.commit().await?;
        Ok(())
    }

    async fn update(&self, user: &User) -> Result<(), anyhow::Error> {
        sqlx::query(
            "UPDATE users SET 
             email = $1, 
             password_hash = $2, 
             updated_at = $3,
             username = $4
             WHERE id = $5"
        )
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(user.updated_at)
        .bind(&user.username)
        .bind(&user.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
