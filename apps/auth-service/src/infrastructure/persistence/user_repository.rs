//! PostgreSQL User Repository
//!
//! 实现 UserRepository trait，使用 PostgreSQL 进行持久化。
//! 
//! 注意：使用 sqlx 的运行时查询 API，避免编译时宏需要数据库连接。

use crate::domain::aggregates::User;
use crate::domain::repositories::{RepositoryError, UserRepository};
use crate::domain::value_objects::{Email, Permission, RoleId, UserId};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

/// PostgreSQL 用户仓储实现
pub struct PgUserRepository {
    pool: Arc<PgPool>,
}

impl PgUserRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn save(&self, user: &mut User) -> Result<(), RepositoryError> {
        // 使用 UPSERT 语句
        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, display_name, avatar_url, is_active, email_verified, created_at, updated_at, last_login_at, tfa_secret, tfa_enabled, tfa_recovery_codes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (id) DO UPDATE SET
                username = EXCLUDED.username,
                email = EXCLUDED.email,
                password_hash = EXCLUDED.password_hash,
                display_name = EXCLUDED.display_name,
                avatar_url = EXCLUDED.avatar_url,
                is_active = EXCLUDED.is_active,
                email_verified = EXCLUDED.email_verified,
                updated_at = EXCLUDED.updated_at,
                last_login_at = EXCLUDED.last_login_at,
                tfa_secret = EXCLUDED.tfa_secret,
                tfa_enabled = EXCLUDED.tfa_enabled,
                tfa_recovery_codes = EXCLUDED.tfa_recovery_codes
            "#,
        )
        .bind(user.id().as_uuid())
        .bind(user.username())
        .bind(user.email().to_string())
        .bind(user.password_hash())
        .bind(user.display_name())
        .bind(user.avatar_url())
        .bind(user.is_active())
        .bind(user.email_verified())
        .bind(user.created_at())
        .bind(user.updated_at())
        .bind(user.last_login_at())
        .bind(user.tfa_secret())
        .bind(user.tfa_enabled())
        .bind(user.tfa_recovery_codes())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        // 更新用户角色关联 - 先删除现有关联
        sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
            .bind(user.id().as_uuid())
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        // 插入新的角色关联
        for role_id in user.roles() {
            sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2)")
                .bind(user.id().as_uuid())
                .bind(role_id.as_uuid())
                .execute(self.pool.as_ref())
                .await
                .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        }

        // TODO: 发布领域事件到 Kafka
        let _events = user.drain_events();

        Ok(())
    }

    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, RepositoryError> {
        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, display_name, avatar_url, is_active, email_verified, 
                   created_at, updated_at, last_login_at, tfa_secret, tfa_enabled, tfa_recovery_codes
            FROM users WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                let roles = self.get_user_roles(id).await?;
                let user_id: uuid::Uuid = row.get("id");
                let username: String = row.get("username");
                let email_str: String = row.get("email");
                let password_hash: String = row.get("password_hash");
                let display_name: String = row.get("display_name");
                let avatar_url: String = row.get("avatar_url");
                let is_active: bool = row.get("is_active");
                let email_verified: bool = row.get("email_verified");
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
                let last_login_at: Option<chrono::DateTime<chrono::Utc>> = row.get("last_login_at");
                let tfa_secret: Option<String> = row.get("tfa_secret");
                let tfa_enabled: bool = row.get("tfa_enabled");
                let tfa_recovery_codes: Vec<String> = row.get::<Option<Vec<String>>, _>("tfa_recovery_codes").unwrap_or_default();

                let email = Email::new(&email_str)
                    .map_err(|e| RepositoryError::SerializationError(e.to_string()))?;

                Ok(Some(User::reconstitute(
                    UserId::from(user_id),
                    username,
                    email,
                    password_hash,
                    display_name,
                    avatar_url,
                    roles,
                    is_active,
                    email_verified,
                    created_at,
                    updated_at,
                    last_login_at,
                    tfa_secret,
                    tfa_enabled,
                    tfa_recovery_codes,
                )))
            }
            None => Ok(None),
        }
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError> {
        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, display_name, avatar_url, is_active, email_verified,
                   created_at, updated_at, last_login_at, tfa_secret, tfa_enabled, tfa_recovery_codes
            FROM users WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                let user_id: uuid::Uuid = row.get("id");
                let user_id = UserId::from(user_id);
                let roles = self.get_user_roles(&user_id).await?;
                let username: String = row.get("username");
                let email_str: String = row.get("email");
                let password_hash: String = row.get("password_hash");
                let display_name: String = row.get("display_name");
                let avatar_url: String = row.get("avatar_url");
                let is_active: bool = row.get("is_active");
                let email_verified: bool = row.get("email_verified");
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
                let last_login_at: Option<chrono::DateTime<chrono::Utc>> = row.get("last_login_at");

                let tfa_secret: Option<String> = row.get("tfa_secret");
                let tfa_enabled: bool = row.get("tfa_enabled");
                let tfa_recovery_codes: Vec<String> = row.get::<Option<Vec<String>>, _>("tfa_recovery_codes").unwrap_or_default();

                let email = Email::new(&email_str)
                    .map_err(|e| RepositoryError::SerializationError(e.to_string()))?;

                Ok(Some(User::reconstitute(
                    user_id,
                    username,
                    email,
                    password_hash,
                    display_name,
                    avatar_url,
                    roles,
                    is_active,
                    email_verified,
                    created_at,
                    updated_at,
                    last_login_at,
                    tfa_secret,
                    tfa_enabled,
                    tfa_recovery_codes,
                )))
            }
            None => Ok(None),
        }
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError> {
        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, display_name, avatar_url, is_active, email_verified,
                   created_at, updated_at, last_login_at, tfa_secret, tfa_enabled, tfa_recovery_codes
            FROM users WHERE email = $1
            "#,
        )
        .bind(email.to_lowercase())
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                let user_id: uuid::Uuid = row.get("id");
                let user_id = UserId::from(user_id);
                let roles = self.get_user_roles(&user_id).await?;
                let username: String = row.get("username");
                let email_str: String = row.get("email");
                let password_hash: String = row.get("password_hash");
                let display_name: String = row.get("display_name");
                let avatar_url: String = row.get("avatar_url");
                let is_active: bool = row.get("is_active");
                let email_verified: bool = row.get("email_verified");
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
                let last_login_at: Option<chrono::DateTime<chrono::Utc>> = row.get("last_login_at");

                let tfa_secret: Option<String> = row.get("tfa_secret");
                let tfa_enabled: bool = row.get("tfa_enabled");
                let tfa_recovery_codes: Vec<String> = row.get::<Option<Vec<String>>, _>("tfa_recovery_codes").unwrap_or_default();

                let email = Email::new(&email_str)
                    .map_err(|e| RepositoryError::SerializationError(e.to_string()))?;

                Ok(Some(User::reconstitute(
                    user_id,
                    username,
                    email,
                    password_hash,
                    display_name,
                    avatar_url,
                    roles,
                    is_active,
                    email_verified,
                    created_at,
                    updated_at,
                    last_login_at,
                    tfa_secret,
                    tfa_enabled,
                    tfa_recovery_codes,
                )))
            }
            None => Ok(None),
        }
    }

    async fn username_exists(&self, username: &str) -> Result<bool, RepositoryError> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE username = $1")
            .bind(username)
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(result.0 > 0)
    }

    async fn email_exists(&self, email: &str) -> Result<bool, RepositoryError> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = $1")
            .bind(email.to_lowercase())
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(result.0 > 0)
    }

    async fn get_user_permissions(&self, id: &UserId) -> Result<Vec<Permission>, RepositoryError> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT p.resource, p.action
            FROM permissions p
            JOIN role_permissions rp ON p.id = rp.permission_id
            JOIN user_roles ur ON rp.role_id = ur.role_id
            WHERE ur.user_id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let resource: String = r.get("resource");
                let action: String = r.get("action");
                Permission::new(resource, action)
            })
            .collect())
    }

    async fn delete(&self, id: &UserId) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id.as_uuid())
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_all(
        &self,
        search: Option<&str>,
        role_id: Option<&RoleId>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<User>, RepositoryError> {
        let mut query = String::from(
            "SELECT u.id, u.username, u.email, u.password_hash, u.display_name, u.avatar_url, u.is_active, u.email_verified, u.created_at, u.updated_at, u.last_login_at, u.tfa_secret, u.tfa_enabled, u.tfa_recovery_codes 
             FROM users u"
        );
        
        let mut conditions = Vec::new();
        if role_id.is_some() {
            query.push_str(" JOIN user_roles ur ON u.id = ur.user_id");
            conditions.push(format!("ur.role_id = ${}", conditions.len() + 1));
        }
        
        if let Some(s) = search {
            conditions.push(format!("(u.username ILIKE ${} OR u.email ILIKE ${} OR u.display_name ILIKE ${})", conditions.len() + 1, conditions.len() + 2, conditions.len() + 3));
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }
        
        query.push_str(&format!(" ORDER BY u.created_at DESC LIMIT ${} OFFSET ${}", conditions.len() + (if search.is_some() { 3 } else { 0 }) + 1, conditions.len() + (if search.is_some() { 3 } else { 0 }) + 2));

        let mut q = sqlx::query(&query);
        
        if let Some(rid) = role_id {
            q = q.bind(rid.as_uuid());
        }
        
        if let Some(s) = search {
            let s_pattern = format!("%{}%", s);
            q = q.bind(s_pattern.clone()).bind(s_pattern.clone()).bind(s_pattern);
        }
        
        q = q.bind(limit).bind(offset);

        let rows = q.fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let mut users = Vec::new();
        for row in rows {
            let user_id: Uuid = row.get("id");
            let username: String = row.get("username");
            let email: String = row.get("email");
            let password_hash: String = row.get("password_hash");
            let display_name: String = row.get("display_name");
            let avatar_url: String = row.get("avatar_url");
            let is_active: bool = row.get("is_active");
            let email_verified: bool = row.get("email_verified");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
            let last_login_at: Option<chrono::DateTime<chrono::Utc>> = row.get("last_login_at");

            // 获取角色
            let roles = self.get_user_roles(&UserId::from(user_id)).await?;

            users.push(User::reconstitute(
                UserId::from(user_id),
                username,
                crate::domain::value_objects::Email::new(&email).unwrap(), // TODO: handle error
                password_hash,
                display_name,
                avatar_url,
                roles,
                is_active,
                email_verified,
                created_at,
                updated_at,
                last_login_at,
                row.get("tfa_secret"),
                row.get("tfa_enabled"),
                row.get::<Option<Vec<String>>, _>("tfa_recovery_codes").unwrap_or_default(),
            ));
        }

        Ok(users)
    }

    async fn count_all(
        &self,
        search: Option<&str>,
        role_id: Option<&RoleId>,
    ) -> Result<i64, RepositoryError> {
        let mut query = String::from("SELECT COUNT(DISTINCT u.id) FROM users u");
        
        let mut conditions = Vec::new();
        if role_id.is_some() {
            query.push_str(" JOIN user_roles ur ON u.id = ur.user_id");
            conditions.push(format!("ur.role_id = ${}", conditions.len() + 1));
        }
        
        if let Some(_) = search {
            conditions.push(format!("(u.username ILIKE ${} OR u.email ILIKE ${} OR u.display_name ILIKE ${})", conditions.len() + 1, conditions.len() + 2, conditions.len() + 3));
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        let mut q = sqlx::query(&query);
        
        if let Some(rid) = role_id {
            q = q.bind(rid.as_uuid());
        }
        
        if let Some(s) = search {
            let s_pattern = format!("%{}%", s);
            q = q.bind(s_pattern.clone()).bind(s_pattern.clone()).bind(s_pattern);
        }

        let row = q.fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(row.get(0))
    }
}

impl PgUserRepository {
    async fn get_user_roles(&self, user_id: &UserId) -> Result<Vec<RoleId>, RepositoryError> {
        let rows = sqlx::query("SELECT role_id FROM user_roles WHERE user_id = $1")
            .bind(user_id.as_uuid())
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let role_id: uuid::Uuid = r.get("role_id");
                RoleId::from(role_id)
            })
            .collect())
    }
}
