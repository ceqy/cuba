use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;
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

        // Parse policy.id as UUID
        let policy_uuid = Uuid::parse_str(&policy.id)
            .map_err(|e| DomainError::InvalidInput(format!("Invalid policy ID: {}", e)))?;

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
        .bind(policy_uuid)
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
        let id_uuid = Uuid::parse_str(id)
            .map_err(|e| DomainError::InvalidInput(format!("Invalid policy ID: {}", e)))?;

        let row = sqlx::query(
            r#"
            SELECT id, name, description, version, statements, created_at, updated_at, tenant_id
            FROM policies
            WHERE id = $1
            "#
        )
        .bind(id_uuid)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        if let Some(row) = row {
            let statements_val: serde_json::Value = row.try_get("statements")
                .map_err(|e| DomainError::InfrastructureError(format!("Failed to get statements: {}", e)))?;

            let statements: Vec<Statement> = serde_json::from_value(statements_val)
                .map_err(|e| DomainError::InternalError(format!("Deserialization error: {}", e)))?;

            let id_uuid: Uuid = row.try_get("id").map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

            Ok(Some(Policy {
                id: id_uuid.to_string(),
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

            let id_uuid: Uuid = row.try_get("id").map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

            Ok(Some(Policy {
                id: id_uuid.to_string(),
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
        let id_uuid = Uuid::parse_str(id)
            .map_err(|e| DomainError::InvalidInput(format!("Invalid policy ID: {}", e)))?;

        sqlx::query("DELETE FROM policies WHERE id = $1")
            .bind(id_uuid)
            .execute(&*self.pool)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(())
    }

    async fn attach_to_role(&self, policy_id: &str, role_id: &str) -> Result<(), DomainError> {
        let policy_uuid = Uuid::parse_str(policy_id)
            .map_err(|e| DomainError::InvalidInput(format!("Invalid policy ID: {}", e)))?;
        let role_uuid = Uuid::parse_str(role_id)
            .map_err(|e| DomainError::InvalidInput(format!("Invalid role ID: {}", e)))?;

        sqlx::query(
            "INSERT INTO role_policies (role_id, policy_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(role_uuid)
        .bind(policy_uuid)
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(())
    }

    async fn attach_to_user(&self, policy_id: &str, user_id: &str) -> Result<(), DomainError> {
        let policy_uuid = Uuid::parse_str(policy_id)
            .map_err(|e| DomainError::InvalidInput(format!("Invalid policy ID: {}", e)))?;
        let user_uuid = Uuid::parse_str(user_id)
            .map_err(|e| DomainError::InvalidInput(format!("Invalid user ID: {}", e)))?;

        sqlx::query(
            "INSERT INTO user_policies (user_id, policy_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(user_uuid)
        .bind(policy_uuid)
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(())
    }

    async fn find_all(&self, limit: i64, offset: i64) -> Result<(Vec<Policy>, i64), DomainError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM policies")
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        let rows = sqlx::query(
            r#"
            SELECT id, name, description, version, statements, created_at, updated_at, tenant_id
            FROM policies
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        let mut policies = Vec::new();
        for row in rows {
            let statements_val: serde_json::Value = row.try_get("statements")
                .map_err(|e| DomainError::InfrastructureError(format!("Failed to get statements: {}", e)))?;

            let statements: Vec<Statement> = serde_json::from_value(statements_val)
                .map_err(|e| DomainError::InternalError(format!("Deserialization error: {}", e)))?;

            let id_uuid: Uuid = row.try_get("id").map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

            policies.push(Policy {
                id: id_uuid.to_string(),
                name: row.try_get("name").map_err(|e| DomainError::InfrastructureError(e.to_string()))?,
                description: row.try_get("description").map_err(|e| DomainError::InfrastructureError(e.to_string()))?,
                version: row.try_get("version").map_err(|e| DomainError::InfrastructureError(e.to_string()))?,
                statements,
                created_at: row.try_get("created_at").map_err(|e| DomainError::InfrastructureError(e.to_string()))?,
                updated_at: row.try_get("updated_at").map_err(|e| DomainError::InfrastructureError(e.to_string()))?,
                tenant_id: row.try_get("tenant_id").map_err(|e| DomainError::InfrastructureError(e.to_string()))?,
            });
        }

        Ok((policies, count))
    }
}
