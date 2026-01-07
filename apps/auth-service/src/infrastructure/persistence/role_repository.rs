//! PostgreSQL Role Repository implementation

use crate::domain::aggregates::Role;
use crate::domain::repositories::{RoleRepository, RepositoryError};
use crate::domain::value_objects::{Permission, RoleId, PermissionId};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

pub struct PgRoleRepository {
    pool: Arc<PgPool>,
}

impl PgRoleRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RoleRepository for PgRoleRepository {
    async fn save(&self, role: &mut Role) -> Result<(), RepositoryError> {
        // 1. Insert or update the role
        sqlx::query(
            r#"
            INSERT INTO roles (id, name, description, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                description = EXCLUDED.description,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(role.id().as_uuid())
        .bind(role.name())
        .bind(role.description())
        .bind(role.created_at())
        .bind(role.updated_at())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        // 2. Handle permissions
        // Sync permissions: delete old associations and insert current ones
        // First, ensure all permissions exist in the permissions table
        for perm in role.permissions() {
            sqlx::query(
                r#"
                INSERT INTO permissions (id, resource, action, created_at)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (resource, action) DO NOTHING
                "#,
            )
            .bind(Uuid::new_v4()) // This ID will be ignored if the permission exists
            .bind(perm.resource())
            .bind(perm.action())
            .bind(chrono::Utc::now())
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        }

        // Remove old role-permission associations
        sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
            .bind(role.id().as_uuid())
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        // Insert new associations
        for perm in role.permissions() {
            sqlx::query(
                r#"
                INSERT INTO role_permissions (role_id, permission_id)
                SELECT $1, id FROM permissions WHERE resource = $2 AND action = $3
                "#,
            )
            .bind(role.id().as_uuid())
            .bind(perm.resource())
            .bind(perm.action())
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        }

        // Drain events (TODO: publish to Kafka)
        let _events = role.drain_events();

        Ok(())
    }

    async fn find_by_id(&self, id: &RoleId) -> Result<Option<Role>, RepositoryError> {
        let row = sqlx::query(
            "SELECT id, name, description, created_at, updated_at FROM roles WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                let role_id: Uuid = row.get("id");
                let name: String = row.get("name");
                let description: Option<String> = row.get("description");
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

                let permissions = self.get_role_permissions(&RoleId::from(role_id)).await?;

                Ok(Some(Role::reconstitute(
                    RoleId::from(role_id),
                    name,
                    description,
                    permissions,
                    created_at,
                    updated_at,
                )))
            }
            None => Ok(None),
        }
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Role>, RepositoryError> {
        let row = sqlx::query(
            "SELECT id, name, description, created_at, updated_at FROM roles WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                let role_id: Uuid = row.get("id");
                let name: String = row.get("name");
                let description: Option<String> = row.get("description");
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

                let permissions = self.get_role_permissions(&RoleId::from(role_id)).await?;

                Ok(Some(Role::reconstitute(
                    RoleId::from(role_id),
                    name,
                    description,
                    permissions,
                    created_at,
                    updated_at,
                )))
            }
            None => Ok(None),
        }
    }

    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<Role>, RepositoryError> {
        let rows = sqlx::query("SELECT id, name, description, created_at, updated_at FROM roles LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let mut roles = Vec::new();
        for row in rows {
            let role_id: Uuid = row.get("id");
            let name: String = row.get("name");
            let description: Option<String> = row.get("description");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

            let permissions = self.get_role_permissions(&RoleId::from(role_id)).await?;

            roles.push(Role::reconstitute(
                RoleId::from(role_id),
                name,
                description,
                permissions,
                created_at,
                updated_at,
            ));
        }

        Ok(roles)
    }

    async fn delete(&self, id: &RoleId) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM roles WHERE id = $1")
            .bind(id.as_uuid())
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_permission_by_id(&self, id: &PermissionId) -> Result<Option<Permission>, RepositoryError> {
        let row = sqlx::query(
            "SELECT resource, action FROM permissions WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                let resource: String = row.get("resource");
                let action: String = row.get("action");
                Ok(Some(Permission::new(resource, action)))
            }
            None => Ok(None),
        }
    }

    async fn find_all_permissions(&self, limit: i64, offset: i64) -> Result<Vec<(PermissionId, Permission)>, RepositoryError> {
        let rows = sqlx::query("SELECT id, resource, action FROM permissions LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let mut permissions = Vec::new();
        for row in rows {
            let id: Uuid = row.get("id");
            let resource: String = row.get("resource");
            let action: String = row.get("action");
            permissions.push((PermissionId::from(id), Permission::new(resource, action)));
        }
        Ok(permissions)
    }

    async fn count_all(&self) -> Result<i64, RepositoryError> {
        let row = sqlx::query("SELECT COUNT(*) FROM roles")
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        
        Ok(row.get::<i64, _>(0))
    }

    async fn count_all_permissions(&self) -> Result<i64, RepositoryError> {
        let row = sqlx::query("SELECT COUNT(*) FROM permissions")
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
            
        Ok(row.get::<i64, _>(0))
    }
}

impl PgRoleRepository {
    async fn get_role_permissions(&self, role_id: &RoleId) -> Result<Vec<Permission>, RepositoryError> {
        let rows = sqlx::query(
            r#"
            SELECT p.resource, p.action
            FROM permissions p
            JOIN role_permissions rp ON p.id = rp.permission_id
            WHERE rp.role_id = $1
            "#,
        )
        .bind(role_id.as_uuid())
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
}
