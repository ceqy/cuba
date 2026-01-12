use crate::domain::entities::{Role, Permission};
use crate::domain::repositories::{RoleRepository, PermissionRepository};
use crate::domain::aggregates::user::UserId;
use async_trait::async_trait;
use cuba_core::repository::Repository;
use cuba_database::DbPool;
use anyhow::{Result, Context};

pub struct PostgresRbacRepository {
    pool: DbPool,
}

impl PostgresRbacRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<Role> for PostgresRbacRepository {
    type Id = String;

    async fn find_by_id(&self, id: &Self::Id) -> Result<Option<Role>> {
        let row: Option<RoleRow> = sqlx::query_as(
            r#"
            SELECT id, name, description, parent_role_id, created_at, updated_at
            FROM roles
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find role by id")?;

        Ok(row.map(|r| r.into_domain()))
    }

    async fn save(&self, entity: &Role) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO roles (id, name, description, parent_role_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE
            SET name = $2, description = $3, parent_role_id = $4, updated_at = $6
            "#,
        )
        .bind(&entity.id)
        .bind(&entity.name)
        .bind(&entity.description)
        .bind(&entity.parent_role_id)
        .bind(entity.created_at)
        .bind(entity.updated_at)
        .execute(&self.pool)
        .await
        .context("Failed to save role")?;

        Ok(())
    }
}

#[async_trait]
impl RoleRepository for PostgresRbacRepository {
    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Vec<Role>> {
        let rows: Vec<RoleRow> = sqlx::query_as(
            r#"
            SELECT r.id, r.name, r.description, r.parent_role_id, r.created_at, r.updated_at
            FROM roles r
            JOIN user_roles ur ON r.id = ur.role_id
            WHERE ur.user_id = $1
            "#,
        )
        .bind(user_id.clone().into_inner())
        .fetch_all(&self.pool)
        .await
        .context("Failed to find roles by user id")?;

        Ok(rows.into_iter().map(|r| r.into_domain()).collect())
    }
}

#[async_trait]
impl Repository<Permission> for PostgresRbacRepository {
    type Id = String;

    async fn find_by_id(&self, id: &Self::Id) -> Result<Option<Permission>> {
        let row: Option<PermissionRow> = sqlx::query_as(
            r#"
            SELECT id, code, resource, action, description, created_at
            FROM permissions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find permission by id")?;

        Ok(row.map(|r| r.into_domain()))
    }

    async fn save(&self, entity: &Permission) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO permissions (id, code, resource, action, description, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE
            SET code = $2, resource = $3, action = $4, description = $5
            "#,
        )
        .bind(&entity.id)
        .bind(&entity.code)
        .bind(&entity.resource)
        .bind(&entity.action)
        .bind(&entity.description)
        .bind(entity.created_at)
        .execute(&self.pool)
        .await
        .context("Failed to save permission")?;

        Ok(())
    }
}

#[async_trait]
impl PermissionRepository for PostgresRbacRepository {
    async fn find_by_role_id(&self, role_id: &str) -> Result<Vec<Permission>> {
        let rows: Vec<PermissionRow> = sqlx::query_as(
            r#"
            SELECT p.id, p.code, p.resource, p.action, p.description, p.created_at
            FROM permissions p
            JOIN role_permissions rp ON p.id = rp.permission_id
            WHERE rp.role_id = $1
            "#,
        )
        .bind(role_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to find permissions by role id")?;

        Ok(rows.into_iter().map(|r| r.into_domain()).collect())
    }

    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Vec<Permission>> {
        let rows: Vec<PermissionRow> = sqlx::query_as(
            r#"
            SELECT DISTINCT p.id, p.code, p.resource, p.action, p.description, p.created_at
            FROM permissions p
            JOIN role_permissions rp ON p.id = rp.permission_id
            JOIN user_roles ur ON rp.role_id = ur.role_id
            WHERE ur.user_id = $1
            "#,
        )
        .bind(user_id.clone().into_inner())
        .fetch_all(&self.pool)
        .await
        .context("Failed to find permissions by user id")?;

        Ok(rows.into_iter().map(|r| r.into_domain()).collect())
    }
}

#[derive(sqlx::FromRow)]
struct RoleRow {
    id: String,
    name: String,
    description: Option<String>,
    parent_role_id: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl RoleRow {
    fn into_domain(self) -> Role {
        Role {
            id: self.id,
            name: self.name,
            description: self.description,
            parent_role_id: self.parent_role_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct PermissionRow {
    id: String,
    code: String,
    resource: String,
    action: String,
    description: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl PermissionRow {
    fn into_domain(self) -> Permission {
        Permission {
            id: self.id,
            code: self.code,
            resource: self.resource,
            action: self.action,
            description: self.description,
            created_at: self.created_at,
        }
    }
}
