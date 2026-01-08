//! Application Services for GL Service
//!
//! 应用层服务，协调领域操作

use chrono::NaiveDate;
use std::sync::Arc;
use tracing::{info, instrument};
use uuid::Uuid;

use crate::domain::entities::{JournalEntry, JournalEntryLine};
use crate::domain::repository::{JournalEntryFilter, JournalEntryRepository, PagedResult, Pagination};
use crate::domain::value_objects::{
    Account, DebitCreditIndicator, DocumentNumber, FiscalPeriod, MonetaryAmount,
};
use crate::domain::rules::JournalEntryStatus;

/// 创建凭证命令
#[derive(Debug)]
pub struct CreateJournalEntryCommand {
    pub company_code: String,
    pub document_type: String,
    pub document_date: NaiveDate,
    pub posting_date: NaiveDate,
    pub fiscal_year: i32,
    pub fiscal_period: i32,
    pub currency: String,
    pub header_text: Option<String>,
    pub lines: Vec<CreateLineItemCommand>,
    pub created_by: Uuid,
}

/// 更新凭证命令
#[derive(Debug)]
pub struct UpdateJournalEntryCommand {
    pub id: Uuid,
    pub document_date: Option<NaiveDate>,
    pub posting_date: Option<NaiveDate>,
    pub header_text: Option<String>,
    pub lines: Option<Vec<CreateLineItemCommand>>,
}

/// 创建行项目命令
#[derive(Debug)]
pub struct CreateLineItemCommand {
    pub gl_account: String,
    pub amount: rust_decimal::Decimal,
    pub debit_credit: String, // "S" or "H"
    pub cost_center: Option<String>,
    pub profit_center: Option<String>,
    pub line_text: Option<String>,
}

/// 凭证应用服务
pub struct JournalEntryService<R: JournalEntryRepository> {
    repository: Arc<R>,
}

impl<R: JournalEntryRepository> JournalEntryService<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// 创建凭证草稿
    #[instrument(skip(self, cmd), fields(company_code = %cmd.company_code))]
    pub async fn create_journal_entry(
        &self,
        cmd: CreateJournalEntryCommand,
    ) -> anyhow::Result<JournalEntry> {
        // 创建会计期间
        let fiscal_period = FiscalPeriod::new(cmd.fiscal_year, cmd.fiscal_period)?;

        // 创建凭证草稿
        let mut entry = JournalEntry::create_draft(
            cmd.company_code.clone(),
            cmd.document_type,
            cmd.document_date,
            cmd.posting_date,
            fiscal_period,
            cmd.currency.clone(),
            cmd.created_by,
        );

        // 添加行项目
        for (idx, line_cmd) in cmd.lines.into_iter().enumerate() {
            let account = Account::gl_account(&line_cmd.gl_account)?;
            let dc_indicator = DebitCreditIndicator::from_str(&line_cmd.debit_credit)
                .ok_or_else(|| anyhow::anyhow!("Invalid debit/credit indicator"))?;
            let amount = MonetaryAmount::new(line_cmd.amount, &cmd.currency, dc_indicator)?;

            let mut line = JournalEntryLine::new((idx + 1) as i32, account, amount);
            
            if let Some(text) = line_cmd.line_text {
                line = line.with_text(&text);
            }

            entry.add_line(line)?;
        }

        // 保存到仓储
        self.repository.save(&mut entry).await?;

        info!(
            id = %entry.id(),
            company_code = %cmd.company_code,
            "Journal entry created"
        );

        Ok(entry)
    }

    /// 根据 ID 查询凭证
    #[instrument(skip(self))]
    pub async fn get_journal_entry(&self, id: Uuid) -> anyhow::Result<Option<JournalEntry>> {
        self.repository.find_by_id(&id).await
    }

    /// 分页查询凭证
    #[instrument(skip(self))]
    pub async fn list_journal_entries(
        &self,
        filter: JournalEntryFilter,
        pagination: Pagination,
    ) -> anyhow::Result<PagedResult<JournalEntry>> {
        self.repository.find_all(filter, pagination).await
    }

    /// 更新凭证
    #[instrument(skip(self, cmd))]
    pub async fn update_journal_entry(
        &self,
        cmd: UpdateJournalEntryCommand,
    ) -> anyhow::Result<JournalEntry> {
        let mut entry = self.repository.find_by_id(&cmd.id).await?
            .ok_or_else(|| anyhow::anyhow!("Journal entry not found: {}", cmd.id))?;

        if !entry.status().is_editable() {
            return Err(anyhow::anyhow!("Cannot update non-editable entry"));
        }

        // 应用更新
        entry.update_header(cmd.header_text, cmd.document_date, cmd.posting_date)
            .map_err(|e| anyhow::anyhow!(e))?;

        if let Some(line_cmds) = cmd.lines {
            let mut new_lines = Vec::new();
            let currency = entry.header().currency.clone();
            
            for (idx, line_cmd) in line_cmds.into_iter().enumerate() {
                let account = Account::gl_account(&line_cmd.gl_account)
                    .map_err(|e| anyhow::anyhow!(e))?;
                let dc_indicator = DebitCreditIndicator::from_str(&line_cmd.debit_credit)
                    .ok_or_else(|| anyhow::anyhow!("Invalid debit/credit indicator"))?;
                let amount = MonetaryAmount::new(line_cmd.amount, &currency, dc_indicator)
                    .map_err(|e| anyhow::anyhow!(e))?;

                let mut line = JournalEntryLine::new((idx + 1) as i32, account, amount);
                if let Some(text) = line_cmd.line_text {
                    line = line.with_text(&text);
                }
                new_lines.push(line);
            }
            entry.replace_lines(new_lines).map_err(|e| anyhow::anyhow!(e))?;
        }

        // 保存到仓储
        self.repository.save(&mut entry).await?;

        Ok(entry)
    }

    /// 删除凭证
    #[instrument(skip(self))]
    pub async fn delete_journal_entry(&self, id: Uuid) -> anyhow::Result<()> {
        let entry = self.repository.find_by_id(&id).await?
            .ok_or_else(|| anyhow::anyhow!("Journal entry not found: {}", id))?;

        if !entry.status().is_editable() {
            return Err(anyhow::anyhow!("Cannot delete non-editable entry"));
        }

        self.repository.delete(&id).await?;
        Ok(())
    }

    /// 过账凭证
    #[instrument(skip(self))]
    pub async fn post_journal_entry(
        &self,
        id: Uuid,
        posted_by: Uuid,
    ) -> anyhow::Result<JournalEntry> {
        // 查询凭证
        let mut entry = self.repository.find_by_id(&id).await?
            .ok_or_else(|| anyhow::anyhow!("Journal entry not found: {}", id))?;

        // 获取凭证号
        let company_code = entry.header().company_code.clone();
        let fiscal_year = entry.header().fiscal_period.year();
        let doc_number = self.repository.next_document_number(&company_code, fiscal_year).await?;
        let document_number = DocumentNumber::new(&company_code, fiscal_year, &doc_number)?;

        // 过账
        entry.post(document_number, posted_by)?;

        // 保存
        self.repository.save(&mut entry).await?;

        info!(
            id = %id,
            document_number = %entry.document_number().map(|d| d.to_string()).unwrap_or_default(),
            "Journal entry posted"
        );

        Ok(entry)
    }
}
