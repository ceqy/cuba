//! PostgreSQL Audit Log Repository implementation

use crate::domain::repositories::{AuditLogRepository, AuditLogData, RepositoryError};
use async_trait::async_trait;
use sqlx::{PgPool, Row, types::Json};
use std::sync::Arc;
use uuid::Uuid;
use std::collections::HashMap;

pub struct PgAuditLogRepository {
    pool: Arc<PgPool>,
}

impl PgAuditLogRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditLogRepository for PgAuditLogRepository {
    async fn save(&self, log: &AuditLogData) -> Result<(), RepositoryError> {
        // 使用之前创建或定义的 events 表，或者创建一个专门的 audit_logs 表
        // 根据之前的迁移，有一个 events 表，但它可能结构不同。
        // 这里假设我们有一个专门的 audit_logs 表，或者我们在 events 表上操作。
        // 为了清晰，我们假设有一个 audit_logs 表。
        
        sqlx::query(
            r#"
            INSERT INTO audit_logs (id, user_id, tenant_id, action, resource, ip_address, user_agent, timestamp, details)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(Uuid::parse_str(&log.id).unwrap_or_else(|_| Uuid::new_v4()))
        .bind(&log.user_id)
        .bind(&log.tenant_id)
        .bind(&log.action)
        .bind(&log.resource)
        .bind(&log.ip_address)
        .bind(&log.user_agent)
        .bind(log.timestamp)
        .bind(Json(&log.details))
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_logs(
        &self,
        user_id: Option<&str>,
        tenant_id: Option<&str>,
        action: Option<&str>,
        start_time: Option<chrono::DateTime<chrono::Utc>>,
        end_time: Option<chrono::DateTime<chrono::Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLogData>, RepositoryError> {
        let mut query = String::from("SELECT id, user_id, tenant_id, action, resource, ip_address, user_agent, timestamp, details FROM audit_logs");
        let mut conditions = Vec::new();
        
        let mut param_idx = 1;
        if user_id.is_some() { conditions.push(format!("user_id = ${}", param_idx)); param_idx +=1; }
        if tenant_id.is_some() { conditions.push(format!("tenant_id = ${}", param_idx)); param_idx +=1; }
        if action.is_some() { conditions.push(format!("action = ${}", param_idx)); param_idx +=1; }
        if start_time.is_some() { conditions.push(format!("timestamp >= ${}", param_idx)); param_idx +=1; }
        if end_time.is_some() { conditions.push(format!("timestamp <= ${}", param_idx)); param_idx +=1; }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }
        
        // LIMIT and OFFSET use next available indices
        query.push_str(&format!(" ORDER BY timestamp DESC LIMIT ${} OFFSET ${}", param_idx, param_idx + 1));

        let mut q = sqlx::query(&query);
        if let Some(uid) = user_id { q = q.bind(uid); }
        if let Some(tid) = tenant_id { q = q.bind(tid); }
        if let Some(act) = action { q = q.bind(act); }
        if let Some(st) = start_time { q = q.bind(st); }
        if let Some(et) = end_time { q = q.bind(et); }
        q = q.bind(limit).bind(offset);

        let rows = q.fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let mut logs = Vec::new();
        for row in rows {
            let details_json: Json<HashMap<String, String>> = row.get("details");
            logs.push(AuditLogData {
                id: row.get::<Uuid, _>("id").to_string(),
                user_id: row.get("user_id"),
                tenant_id: row.get::<Option<String>, _>("tenant_id").unwrap_or_default(),
                action: row.get("action"),
                resource: row.get("resource"),
                ip_address: row.get("ip_address"),
                user_agent: row.get("user_agent"),
                timestamp: row.get("timestamp"),
                details: details_json.0,
            });
        }
        Ok(logs)
    }

    async fn count_logs(
        &self,
        user_id: Option<&str>,
        tenant_id: Option<&str>,
        action: Option<&str>,
        start_time: Option<chrono::DateTime<chrono::Utc>>,
        end_time: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<i64, RepositoryError> {
        let mut query = String::from("SELECT COUNT(*) FROM audit_logs");
        let mut conditions = Vec::new();
        
        let mut param_idx = 1;
        if user_id.is_some() { conditions.push(format!("user_id = ${}", param_idx)); param_idx +=1; }
        if tenant_id.is_some() { conditions.push(format!("tenant_id = ${}", param_idx)); param_idx +=1; }
        if action.is_some() { conditions.push(format!("action = ${}", param_idx)); param_idx +=1; }
        if start_time.is_some() { conditions.push(format!("timestamp >= ${}", param_idx)); param_idx +=1; }
        if end_time.is_some() { conditions.push(format!("timestamp <= ${}", param_idx)); param_idx +=1; }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        let mut q = sqlx::query(&query);
        if let Some(uid) = user_id { q = q.bind(uid); }
        if let Some(tid) = tenant_id { q = q.bind(tid); }
        if let Some(act) = action { q = q.bind(act); }
        if let Some(st) = start_time { q = q.bind(st); }
        if let Some(et) = end_time { q = q.bind(et); }

        let row = q.fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(row.get(0))
    }
}
