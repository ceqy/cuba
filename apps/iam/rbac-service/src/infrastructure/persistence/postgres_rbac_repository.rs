use crate::domain::repositories::{PermissionRepository, RoleRepository};
use crate::domain::{Permission, Role};
use async_trait::async_trait;
use cuba_database::DbPool;
use sqlx::Row;

pub struct PostgresRbacRepository {
    pool: DbPool,
}

impl PostgresRbacRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RoleRepository for PostgresRbacRepository {
    async fn find_all(&self) -> Result<Vec<Role>, anyhow::Error> {
        let rows = sqlx::query("SELECT * FROM roles")
            .fetch_all(&self.pool)
            .await?;
        let mut roles = Vec::new();
        for row in rows {
            roles.push(Role {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                description: row.try_get("description")?,
                parent_id: row.try_get("parent_id").ok(),
                tenant_id: row.try_get("tenant_id")?,
                is_immutable: row.try_get("is_immutable").unwrap_or(false),
                created_at: row.try_get("created_at")?,
            });
        }
        Ok(roles)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Role>, anyhow::Error> {
        let row = sqlx::query("SELECT * FROM roles WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| Role {
            id: r.try_get("id").unwrap(),
            name: r.try_get("name").unwrap(),
            description: r.try_get("description").unwrap(),
            parent_id: r.try_get("parent_id").ok(),
            tenant_id: r.try_get("tenant_id").unwrap(),
            is_immutable: r.try_get("is_immutable").unwrap_or(false),
            created_at: r.try_get("created_at").unwrap(),
        }))
    }

    async fn save(&self, role: &Role) -> Result<(), anyhow::Error> {
        sqlx::query(
            "INSERT INTO roles (id, name, description, parent_id, tenant_id, is_immutable, created_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name, description = EXCLUDED.description"
        )
        .bind(&role.id)
        .bind(&role.name)
        .bind(&role.description)
        .bind(&role.parent_id)
        .bind(&role.tenant_id)
        .bind(role.is_immutable)
        .bind(role.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM roles WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_by_user_id(&self, user_id: &str) -> anyhow::Result<Vec<Role>> {
        let rows = sqlx::query(
            "WITH RECURSIVE role_hierarchy AS (
                SELECT role_id FROM user_roles WHERE user_id = $1
                UNION
                SELECT r.parent_id FROM roles r
                JOIN role_hierarchy rh ON r.id = rh.role_id
                WHERE r.parent_id IS NOT NULL
            )
            SELECT r.* FROM roles r 
            JOIN role_hierarchy rh ON r.id = rh.role_id",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut roles = Vec::new();
        for row in rows {
            roles.push(Role {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                description: row.try_get("description")?,
                parent_id: row.try_get("parent_id").ok(),
                tenant_id: row.try_get("tenant_id")?,
                is_immutable: row.try_get("is_immutable").unwrap_or(false),
                created_at: row.try_get("created_at")?,
            });
        }
        Ok(roles)
    }

    async fn grant_permissions(
        &self,
        role_id: &str,
        permission_ids: &[String],
    ) -> anyhow::Result<()> {
        for perm_id in permission_ids {
            sqlx::query("INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(role_id)
                .bind(perm_id)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    async fn assign_to_user(&self, user_id: &str, role_id: &str) -> Result<(), anyhow::Error> {
        sqlx::query(
            "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(user_id)
        .bind(role_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn remove_from_user(&self, user_id: &str, role_id: &str) -> Result<(), anyhow::Error> {
        sqlx::query("DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2")
            .bind(user_id)
            .bind(role_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl PermissionRepository for PostgresRbacRepository {
    async fn find_all(&self) -> Result<Vec<Permission>, anyhow::Error> {
        let rows = sqlx::query("SELECT * FROM permissions")
            .fetch_all(&self.pool)
            .await?;
        let mut perms = Vec::new();
        for row in rows {
            perms.push(Permission {
                id: row.try_get("id")?,
                code: row.try_get("code")?,
                resource: row.try_get("resource")?,
                action: row.try_get("action")?,
                description: row.try_get("description")?,
                created_at: row.try_get("created_at")?,
            });
        }
        Ok(perms)
    }

    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Permission>, anyhow::Error> {
        let rows = sqlx::query(
            "WITH RECURSIVE role_hierarchy AS (
                SELECT role_id FROM user_roles WHERE user_id = $1
                UNION
                SELECT r.parent_id FROM roles r
                JOIN role_hierarchy rh ON r.id = rh.role_id
                WHERE r.parent_id IS NOT NULL
            )
            SELECT DISTINCT p.* FROM permissions p 
            JOIN role_permissions rp ON p.id = rp.permission_id 
            JOIN role_hierarchy rh ON rp.role_id = rh.role_id",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        let mut perms = Vec::new();
        for row in rows {
            perms.push(Permission {
                id: row.try_get("id")?,
                code: row.try_get("code")?,
                resource: row.try_get("resource")?,
                action: row.try_get("action")?,
                description: row.try_get("description")?,
                created_at: row.try_get("created_at")?,
            });
        }
        Ok(perms)
    }
}
