//! Batch Processing Module for GL Service
//!
//! 批量处理服务实现

use chrono::{Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, instrument};
use uuid::Uuid;
use std::sync::Arc;

use crate::domain::repository::JournalEntryRepository;
use crate::application::{CreateJournalEntryCommand, CreateLineItemCommand, JournalEntryService};

// ============================================================================
// Batch Upload (批量上传)
// ============================================================================

/// 批量上传项目
#[derive(Debug, Clone, Deserialize)]
pub struct BatchUploadItem {
    pub company_code: String,
    pub document_type: String,
    pub document_date: NaiveDate,
    pub posting_date: NaiveDate,
    pub currency: String,
    pub header_text: Option<String>,
    pub lines: Vec<BatchLineItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchLineItem {
    pub gl_account: String,
    pub amount: Decimal,
    pub debit_credit: String,
    pub cost_center: Option<String>,
    pub profit_center: Option<String>,
    pub tax_code: Option<String>,
}

/// 批量上传结果
#[derive(Debug, Clone, Serialize)]
pub struct BatchUploadResult {
    pub total_count: usize,
    pub success_count: usize,
    pub failed_count: usize,
    pub results: Vec<BatchItemResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchItemResult {
    pub index: usize,
    pub success: bool,
    pub document_number: Option<String>,
    pub error_message: Option<String>,
}

// ============================================================================
// Period Close (期间关账)
// ============================================================================

/// 期间关账请求
#[derive(Debug, Clone, Deserialize)]
pub struct PeriodCloseRequest {
    pub company_code: String,
    pub fiscal_year: i32,
    pub period: i32,
    pub close_type: PeriodCloseType,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub enum PeriodCloseType {
    /// 软关账 - 仅标记，仍可过账
    Soft,
    /// 硬关账 - 禁止所有过账
    Hard,
}

/// 期间关账结果
#[derive(Debug, Clone, Serialize)]
pub struct PeriodCloseResult {
    pub company_code: String,
    pub fiscal_year: i32,
    pub period: i32,
    pub close_type: PeriodCloseType,
    pub closed_at: String,
    pub open_items_count: i32,
}

// ============================================================================
// Recurring Entries (周期性凭证)
// ============================================================================

/// 周期性凭证模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringEntryTemplate {
    pub id: Uuid,
    pub name: String,
    pub company_code: String,
    pub document_type: String,
    pub frequency: RecurringFrequency,
    pub next_run_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub template_lines: Vec<RecurringTemplateLine>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RecurringFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringTemplateLine {
    pub gl_account: String,
    pub amount: Decimal,
    pub debit_credit: String,
    pub cost_center: Option<String>,
}

// ============================================================================
// Batch Service
// ============================================================================

pub struct BatchService<R: JournalEntryRepository> {
    journal_service: Arc<JournalEntryService<R>>,
}

impl<R: JournalEntryRepository> BatchService<R> {
    pub fn new(journal_service: Arc<JournalEntryService<R>>) -> Self {
        Self { journal_service }
    }

    /// 批量上传凭证
    #[instrument(skip(self, items))]
    pub async fn batch_upload(
        &self,
        items: Vec<BatchUploadItem>,
        created_by: Uuid,
    ) -> anyhow::Result<BatchUploadResult> {
        let total_count = items.len();
        let mut success_count = 0;
        let mut failed_count = 0;
        let mut results = Vec::with_capacity(total_count);

        for (index, item) in items.into_iter().enumerate() {
            let command = CreateJournalEntryCommand {
                company_code: item.company_code,
                document_type: item.document_type,
                document_date: item.document_date,
                posting_date: item.posting_date,
                fiscal_year: item.posting_date.year() as i32,
                fiscal_period: item.posting_date.month() as i32,
                currency: item.currency,
                header_text: item.header_text,
                lines: item.lines.into_iter().map(|l| CreateLineItemCommand {
                    gl_account: l.gl_account,
                    amount: l.amount,
                    debit_credit: l.debit_credit,
                    cost_center: l.cost_center,
                    profit_center: l.profit_center,
                    line_text: None,
                    tax_code: l.tax_code,
                }).collect(),
                created_by,
                exchange_rate: None,
            };

            match self.journal_service.create_journal_entry(command).await {
                Ok(entry) => {
                    success_count += 1;
                    results.push(BatchItemResult {
                        index,
                        success: true,
                        document_number: entry.document_number().map(|d| d.number().to_string()),
                        error_message: None,
                    });
                }
                Err(e) => {
                    failed_count += 1;
                    warn!(index = index, error = %e, "Batch upload item failed");
                    results.push(BatchItemResult {
                        index,
                        success: false,
                        document_number: None,
                        error_message: Some(e.to_string()),
                    });
                }
            }
        }

        info!(
            total = total_count,
            success = success_count,
            failed = failed_count,
            "Batch upload completed"
        );

        Ok(BatchUploadResult {
            total_count,
            success_count,
            failed_count,
            results,
        })
    }

    /// 期间关账 (占位实现)
    #[instrument(skip(self))]
    pub async fn close_period(
        &self,
        request: PeriodCloseRequest,
    ) -> anyhow::Result<PeriodCloseResult> {
        info!(
            company_code = %request.company_code,
            fiscal_year = request.fiscal_year,
            period = request.period,
            "Closing period"
        );

        // TODO: 实现实际的期间关账逻辑
        // 1. 检查是否有未过账凭证
        // 2. 更新期间状态表
        // 3. 记录关账日志

        Ok(PeriodCloseResult {
            company_code: request.company_code,
            fiscal_year: request.fiscal_year,
            period: request.period,
            close_type: request.close_type,
            closed_at: Utc::now().to_rfc3339(),
            open_items_count: 0,
        })
    }
}
