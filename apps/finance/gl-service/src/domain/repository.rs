//! Repository trait for GL Service
//!
//! 仓储接口定义

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::{JournalEntry, JournalEntryLine, ClearingDocument};
use crate::domain::value_objects::DocumentNumber;
use crate::domain::rules::JournalEntryStatus;

/// 分页参数
#[derive(Debug, Clone)]
pub struct Pagination {
    pub page: u32,
    pub page_size: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self { page: 1, page_size: 20 }
    }
}

/// 凭证查询过滤器
#[derive(Debug, Clone, Default)]
pub struct JournalEntryFilter {
    pub company_code: Option<String>,
    pub fiscal_year: Option<i32>,
    pub fiscal_period: Option<i32>,
    pub status: Option<JournalEntryStatus>,
    pub created_by: Option<Uuid>,
    pub posting_date_from: Option<chrono::NaiveDate>,
    pub posting_date_to: Option<chrono::NaiveDate>,
}

/// 分页结果
#[derive(Debug)]
pub struct PagedResult<T> {
    pub items: Vec<T>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
}

impl<T> PagedResult<T> {
    pub fn total_pages(&self) -> u32 {
        ((self.total_count as f64) / (self.page_size as f64)).ceil() as u32
    }
}

/// 凭证仓储 trait
#[async_trait]
pub trait JournalEntryRepository: Send + Sync {
    /// 保存凭证 (创建或更新)
    async fn save(&self, entry: &mut JournalEntry) -> anyhow::Result<()>;
    
    /// 根据 ID 查找凭证
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<JournalEntry>>;
    
    /// 根据凭证号查找
    async fn find_by_document_number(&self, doc_number: &DocumentNumber) -> anyhow::Result<Option<JournalEntry>>;
    
    /// 分页查询凭证
    async fn find_all(
        &self,
        filter: JournalEntryFilter,
        pagination: Pagination,
    ) -> anyhow::Result<PagedResult<JournalEntry>>;
    
    /// 删除凭证
    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool>;
    
    /// 生成下一个凭证号
    async fn next_document_number(
        &self,
        company_code: &str,
        fiscal_year: i32,
    ) -> anyhow::Result<String>;

    /// 批量查找行项目
    async fn find_lines_by_ids(&self, ids: &[Uuid]) -> anyhow::Result<Vec<JournalEntryLine>>;

    /// 保存清账凭证并更新关联行项目状态
    async fn save_clearing_document(&self, clearing_doc: &ClearingDocument) -> anyhow::Result<()>;
}
