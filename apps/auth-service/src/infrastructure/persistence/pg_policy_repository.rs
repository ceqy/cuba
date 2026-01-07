use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use crate::domain::aggregates::{Policy, policy::Statement};
use crate::domain::errors::DomainError;
use crate::domain::repositories::PolicyRepository;

pub struct PgPolicyRepository {
    pool: Arc<PgPool>,
}

impl PgPolicyRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PolicyRepository for PgPolicyRepository {
    async fn save(&self, policy: &Policy) -> Result<Policy, DomainError> {
        let statements_json = serde_json::to_value(&policy.statements)
            .map_err(|e| DomainError::InternalError(format!("Serialization error: {}", e)))?;

        sqlx::query(
            r#"
            INSERT INTO policies (id, name, description, version, statements, created_at, updated_at, tenant_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE
            SET name = $2,
                description = $3,
                version = $4,
                statements = $5,
                updated_at = $7
            "#
        )
        .bind(&policy.id)
        .bind(&policy.name)
        .bind(&policy.description)
        .bind(&policy.version)
        .bind(&statements_json)
        .bind(policy.created_at)
        .bind(policy.updated_at)
        .bind(&policy.tenant_id)
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(policy.clone())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Policy>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, name, description, version, statements, created_at, updated_at, tenant_id
            FROM policies
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        if let Some(row) = row {
            let statements_val: serde_json::Value = row.try_get("statements")
                .map_err(|e| DomainError::InfrastructureError(format!("Failed to get statements: {}", e)))?;

            let statements: Vec<Statement> = serde_json::from_value(statements_val)
                .map_err(|e| DomainError::InternalError(format!("Deserialization error: {}", e)))?;

            Ok(Some(Policy {
                id: row.try_get("id").map_err(|e| DomainError::InfrastructureError(e.to_string()))?,
                name: row.try_get("name").map_err(|e| DomainError::InfrastructureError(e.to_string()))?,
                description: row.try_get("description").map_err(|e| DomainError::InfrastructureError(e.to_string()))?,
                version: row.try_get("version").map_err(|e| DomainError::InfrastructureError(e.to_string()))?,
                statements,
                created_at: row.try_get("created_at").map_err(|e| DomainError::InfrastructureError(e.to_string()))?,
                updated_at: row.try_get("updated_at").map_err(|e| DomainError::InfrastructureError(e.to_string()))?,
                tenant_id: row.try_get("tenant_id").map_err(|e| DomainError::InfrastructureError(e.to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn find_by_name(&self, name: &str, tenant_id: &str) -> Result<Option<Policy>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, name, description, version, statements, created_at, updated_at, tenant_id
            FROM policies
            WHERE name = $1 AND tenant_id = $2
            "#
        )
        .bind(name)
        .bind(tenant_id)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        if let Some(row) = row {
            let statements_val: serde_json::Value = row.try_get("statements")
                .map_err(|e| DomainError::InfrastructureError(format!("Failed to get statements: {}", e)))?;

            let statements: Vec<Statement> = serde_json::from_value(statements_val)
                .map_err(|e| DomainError::InternalError(format!("Deserialization error: {}", e)))?;

            Ok(Some(Policy {
                id: row.try_get("id").map_err(|e| DomainError::InfrastructureError(e.to_string()))?,
                name: row.try_get("name").map_err(|e| DomainError::InfrastructureError(e.to_string()))?,
                description: row.try_get("description").map_err(|e| DomainError::InfrastructureError(e.to_string()))?,
                version: row.try_get("version").map_err(|e| DomainError::InfrastructureError(e.to_string()))?,
                statements,
                created_at: row.try_get("created_at").map_err(|e| DomainError::InfrastructureError(e.to_string()))?,
                updated_at: row.try_get("updated_at").map_err(|e| DomainError::InfrastructureError(e.to_string()))?,
                tenant_id: row.try_get("tenant_id").map_err(|e| DomainError::InfrastructureError(e.to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM policies WHERE id = $1")
            .bind(id)
            .execute(&*self.pool)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(())
    }

    async fn attach_to_role(&self, policy_id: &str, role_id: &str) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO role_policies (role_id, policy_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(role_id)
        .bind(policy_id)
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(())
    }

    async fn attach_to_user(&self, policy_id: &str, user_id: &str) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO user_policies (user_id, policy_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(user_id)
        .bind(policy_id)
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(())
    }
}
