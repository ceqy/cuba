//! Infrastructure persistence layer for GL Service
//!
//! PostgreSQL 仓储实现

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entities::JournalEntry;
use crate::domain::repository::{
    JournalEntryFilter, JournalEntryRepository, PagedResult, Pagination,
};
use crate::domain::value_objects::DocumentNumber;

pub struct PgJournalEntryRepository {
    pool: Arc<PgPool>,
}

impl PgJournalEntryRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JournalEntryRepository for PgJournalEntryRepository {
    async fn save(&self, _entry: &mut JournalEntry) -> anyhow::Result<()> {
        // TODO: 实现保存逻辑
        Ok(())
    }

    async fn find_by_id(&self, _id: &Uuid) -> anyhow::Result<Option<JournalEntry>> {
        // TODO: 实现查询逻辑
        Ok(None)
    }

    async fn find_by_document_number(
        &self,
        _doc_number: &DocumentNumber,
    ) -> anyhow::Result<Option<JournalEntry>> {
        // TODO: 实现查询逻辑
        Ok(None)
    }

    async fn find_all(
        &self,
        _filter: JournalEntryFilter,
        pagination: Pagination,
    ) -> anyhow::Result<PagedResult<JournalEntry>> {
        // TODO: 实现分页查询
        Ok(PagedResult {
            items: Vec::new(),
            total_count: 0,
            page: pagination.page,
            page_size: pagination.page_size,
        })
    }

    async fn delete(&self, _id: &Uuid) -> anyhow::Result<bool> {
        // TODO: 实现删除逻辑
        Ok(false)
    }

    async fn next_document_number(
        &self,
        company_code: &str,
        fiscal_year: i32,
    ) -> anyhow::Result<String> {
        // TODO: 实现凭证号生成
        // 暂时返回简单格式
        let timestamp = chrono::Utc::now().timestamp_millis() % 1000000;
        Ok(format!("{:010}", timestamp))
    }
}
