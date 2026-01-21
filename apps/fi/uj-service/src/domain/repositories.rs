use crate::domain::aggregates::UniversalJournalEntry;
use async_trait::async_trait;
use chrono::NaiveDate;

// 定义错误类型
pub type RepositoryError = Box<dyn std::error::Error + Send + Sync>;

/// Universal Journal 查询过滤器
#[derive(Debug, Clone, Default)]
pub struct UniversalJournalFilter {
    // 主键过滤
    pub ledgers: Option<Vec<String>>,
    pub company_codes: Option<Vec<String>>,
    pub fiscal_year_from: Option<i32>,
    pub fiscal_year_to: Option<i32>,
    pub document_types: Option<Vec<String>>,

    // 日期过滤
    pub posting_date_from: Option<NaiveDate>,
    pub posting_date_to: Option<NaiveDate>,
    pub document_date_from: Option<NaiveDate>,
    pub document_date_to: Option<NaiveDate>,

    // 科目过滤
    pub gl_accounts: Option<Vec<String>>,
    pub account_types: Option<Vec<String>>,
    pub business_partners: Option<Vec<String>>,

    // 成本对象过滤
    pub cost_centers: Option<Vec<String>>,
    pub profit_centers: Option<Vec<String>>,
    pub segments: Option<Vec<String>>,
    pub business_areas: Option<Vec<String>>,

    // 来源模块过滤
    pub source_modules: Option<Vec<String>>,

    // 清账状态过滤
    pub only_open_items: bool,
    pub only_cleared_items: bool,

    // 特殊总账过滤
    pub special_gl_indicators: Option<Vec<String>>,

    // 全文搜索
    pub search_text: Option<String>,
}

/// 分页参数
#[derive(Debug, Clone)]
pub struct PaginationParams {
    pub page: i64,
    pub page_size: i64,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 50,
        }
    }
}

impl PaginationParams {
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.page_size
    }

    pub fn limit(&self) -> i64 {
        self.page_size
    }
}

/// 分页响应
#[derive(Debug, Clone)]
pub struct PaginationResponse {
    pub total_count: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

impl PaginationResponse {
    pub fn new(total_count: i64, page: i64, page_size: i64) -> Self {
        let total_pages = (total_count + page_size - 1) / page_size;
        Self {
            total_count,
            page,
            page_size,
            total_pages,
        }
    }
}

/// Universal Journal Repository 接口
#[async_trait]
pub trait UniversalJournalRepository: Send + Sync {
    /// 查询 Universal Journal 条目（分页）
    async fn query(
        &self,
        filter: &UniversalJournalFilter,
        pagination: &PaginationParams,
        order_by: &[String],
    ) -> Result<(Vec<UniversalJournalEntry>, PaginationResponse), RepositoryError>;

    /// 流式查询 Universal Journal 条目
    async fn stream(
        &self,
        filter: &UniversalJournalFilter,
        order_by: &[String],
    ) -> Result<Vec<UniversalJournalEntry>, RepositoryError>;

    /// 分批流式查询 Universal Journal 条目
    async fn stream_batched(
        &self,
        filter: &UniversalJournalFilter,
        order_by: &[String],
        params: &crate::domain::streaming::StreamingParams,
    ) -> Result<std::pin::Pin<Box<dyn futures::Stream<Item = Result<Vec<UniversalJournalEntry>, RepositoryError>> + Send>>, RepositoryError>;


    /// 获取单条 Universal Journal 记录
    async fn get_by_key(
        &self,
        ledger: &str,
        company_code: &str,
        fiscal_year: i32,
        document_number: &str,
        document_line: i32,
    ) -> Result<Option<UniversalJournalEntry>, RepositoryError>;

    /// 聚合查询
    async fn aggregate(
        &self,
        filter: &UniversalJournalFilter,
        dimensions: &[String],
        measure: &str,
        measure_field: &str,
    ) -> Result<Vec<AggregationResult>, RepositoryError>;

    /// 保存 Universal Journal 条目（用于数据同步）
    async fn save(&self, entry: &UniversalJournalEntry) -> Result<(), RepositoryError>;

    /// 批量保存 Universal Journal 条目
    async fn batch_save(&self, entries: &[UniversalJournalEntry]) -> Result<(), RepositoryError>;
}

/// 聚合结果
#[derive(Debug, Clone)]
pub struct AggregationResult {
    pub dimension_values: std::collections::HashMap<String, String>,
    pub measure_value: String,
    pub record_count: i64,
}
