//! 状态历史 Repository trait 定义

use crate::StatusHistory;
use async_trait::async_trait;
use uuid::Uuid;

/// 状态历史 Repository 错误
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("数据库错误: {0}")]
    Database(String),
    #[error("记录不存在")]
    NotFound,
}

impl From<sqlx::Error> for RepositoryError {
    fn from(err: sqlx::Error) -> Self {
        RepositoryError::Database(err.to_string())
    }
}

/// 状态历史 Repository trait
/// 各模块可使用此 trait 实现自己的 Repository
#[async_trait]
pub trait StatusHistoryRepository: Send + Sync {
    /// 保存状态历史记录
    async fn save(&self, history: &StatusHistory) -> Result<(), RepositoryError>;

    /// 查询单据的所有状态历史（按时间升序）
    async fn find_by_document_id(
        &self,
        document_id: Uuid,
    ) -> Result<Vec<StatusHistory>, RepositoryError>;

    /// 查询单据最近一次状态变更
    async fn find_latest_by_document_id(
        &self,
        document_id: Uuid,
    ) -> Result<Option<StatusHistory>, RepositoryError>;

    /// 查询特定操作类型的历史
    async fn find_by_action_type(
        &self,
        document_id: Uuid,
        action_type: &str,
    ) -> Result<Vec<StatusHistory>, RepositoryError>;

    /// 查询特定单据类型的所有历史（用于报表）
    async fn find_by_document_type(
        &self,
        document_type: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StatusHistory>, RepositoryError>;
}

/// PostgreSQL 实现的状态历史 Repository
/// 使用通用表结构，支持多种单据类型
pub struct PgStatusHistoryRepository {
    pool: sqlx::PgPool,
    table_name: String,
}

impl PgStatusHistoryRepository {
    /// 创建新的 Repository 实例
    ///
    /// # Arguments
    /// * `pool` - 数据库连接池
    /// * `table_name` - 状态历史表名（如 "invoice_status_history"）
    pub fn new(pool: sqlx::PgPool, table_name: impl Into<String>) -> Self {
        Self {
            pool,
            table_name: table_name.into(),
        }
    }
}

#[async_trait]
impl StatusHistoryRepository for PgStatusHistoryRepository {
    async fn save(&self, history: &StatusHistory) -> Result<(), RepositoryError> {
        let query = format!(
            r#"
            INSERT INTO {} (
                id, document_id, document_type, from_status, to_status,
                reason, action_type, changed_by, changed_by_name,
                changed_at, remarks, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            self.table_name
        );

        sqlx::query(&query)
            .bind(history.id)
            .bind(history.document_id)
            .bind(&history.document_type)
            .bind(&history.from_status)
            .bind(&history.to_status)
            .bind(&history.reason)
            .bind(&history.action_type)
            .bind(&history.changed_by)
            .bind(&history.changed_by_name)
            .bind(history.changed_at)
            .bind(&history.remarks)
            .bind(&history.metadata)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn find_by_document_id(
        &self,
        document_id: Uuid,
    ) -> Result<Vec<StatusHistory>, RepositoryError> {
        let query = format!(
            r#"
            SELECT id, document_id, document_type, from_status, to_status,
                   reason, action_type, changed_by, changed_by_name,
                   changed_at, remarks, metadata
            FROM {}
            WHERE document_id = $1
            ORDER BY changed_at ASC
            "#,
            self.table_name
        );

        let rows = sqlx::query_as::<_, StatusHistory>(&query)
            .bind(document_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }

    async fn find_latest_by_document_id(
        &self,
        document_id: Uuid,
    ) -> Result<Option<StatusHistory>, RepositoryError> {
        let query = format!(
            r#"
            SELECT id, document_id, document_type, from_status, to_status,
                   reason, action_type, changed_by, changed_by_name,
                   changed_at, remarks, metadata
            FROM {}
            WHERE document_id = $1
            ORDER BY changed_at DESC
            LIMIT 1
            "#,
            self.table_name
        );

        let row = sqlx::query_as::<_, StatusHistory>(&query)
            .bind(document_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row)
    }

    async fn find_by_action_type(
        &self,
        document_id: Uuid,
        action_type: &str,
    ) -> Result<Vec<StatusHistory>, RepositoryError> {
        let query = format!(
            r#"
            SELECT id, document_id, document_type, from_status, to_status,
                   reason, action_type, changed_by, changed_by_name,
                   changed_at, remarks, metadata
            FROM {}
            WHERE document_id = $1 AND action_type = $2
            ORDER BY changed_at ASC
            "#,
            self.table_name
        );

        let rows = sqlx::query_as::<_, StatusHistory>(&query)
            .bind(document_id)
            .bind(action_type)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }

    async fn find_by_document_type(
        &self,
        document_type: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StatusHistory>, RepositoryError> {
        let query = format!(
            r#"
            SELECT id, document_id, document_type, from_status, to_status,
                   reason, action_type, changed_by, changed_by_name,
                   changed_at, remarks, metadata
            FROM {}
            WHERE document_type = $1
            ORDER BY changed_at DESC
            LIMIT $2 OFFSET $3
            "#,
            self.table_name
        );

        let rows = sqlx::query_as::<_, StatusHistory>(&query)
            .bind(document_type)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }
}

/// 创建状态历史表的 SQL（供 Migration 使用）
pub fn create_table_sql(table_name: &str) -> String {
    format!(
        r#"
CREATE TABLE IF NOT EXISTS {} (
    id UUID PRIMARY KEY,
    document_id UUID NOT NULL,
    document_type VARCHAR(50) NOT NULL,
    from_status VARCHAR(50),
    to_status VARCHAR(50) NOT NULL,
    reason TEXT,
    action_type VARCHAR(50) NOT NULL,
    changed_by VARCHAR(100),
    changed_by_name VARCHAR(200),
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    remarks TEXT,
    metadata JSONB
);

CREATE INDEX IF NOT EXISTS idx_{0}_document_id ON {0}(document_id);
CREATE INDEX IF NOT EXISTS idx_{0}_document_type ON {0}(document_type);
CREATE INDEX IF NOT EXISTS idx_{0}_action_type ON {0}(action_type);
CREATE INDEX IF NOT EXISTS idx_{0}_changed_at ON {0}(changed_at DESC);

COMMENT ON TABLE {} IS '单据状态变更历史 - 审计追踪';
"#,
        table_name, table_name
    )
}
