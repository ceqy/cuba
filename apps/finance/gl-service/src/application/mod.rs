//! Application Services for GL Service
//!
//! 应用层服务，协调领域操作

pub mod authorization;
pub mod reports;

use chrono::{Utc, NaiveDate};
use std::sync::Arc;
use tracing::{info, instrument};
use uuid::Uuid;

use crate::domain::entities::{JournalEntry, JournalEntryLine};
use crate::domain::repository::{JournalEntryFilter, JournalEntryRepository, PagedResult, Pagination};
use crate::domain::value_objects::{
    Account, DebitCreditIndicator, DocumentNumber, FiscalPeriod, MonetaryAmount, JournalEntryId,
    ExchangeRate,
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
    pub exchange_rate: Option<rust_decimal::Decimal>,
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
    pub tax_code: Option<String>,
}

/// 模拟税务服务
pub struct MockTaxService;

impl crate::domain::entities::TaxService for MockTaxService {
    fn get_tax_info(&self, tax_code: &str) -> Option<(crate::domain::value_objects::TaxType, rust_decimal::Decimal)> {
        use crate::domain::value_objects::TaxType;
        use rust_decimal_macros::dec;
        match tax_code {
            "V1" | "J1" => Some((TaxType::Input, dec!(0.13))), // 13% 进项税
            "V2" | "J2" => Some((TaxType::Input, dec!(0.09))), // 9% 进项税
            "S1" | "X1" => Some((TaxType::Output, dec!(0.13))), // 13% 销项税
            _ => None,
        }
    }
}

/// 凭证应用服务
/// 清账命令
#[derive(Debug)]
pub struct ClearOpenItemsCommand {
    pub company_code: String,
    pub fiscal_year: i32,
    pub line_ids: Vec<Uuid>,
    pub clearing_date: NaiveDate,
    pub currency: String,
    pub created_by: Uuid,
}

pub struct JournalEntryService<R: JournalEntryRepository> {
    repository: Arc<R>,
}

impl<R: JournalEntryRepository> JournalEntryService<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// 清账 (核销未清项)
    #[instrument(skip(self))]
    pub async fn clear_open_items(
        &self,
        command: ClearOpenItemsCommand,
    ) -> anyhow::Result<crate::domain::entities::ClearingDocument> {
        use crate::domain::entities::ClearingDocument;
        
        // 1. 查询所有待清行项目
        let lines = self.repository.find_lines_by_ids(&command.line_ids).await?;
        if lines.len() != command.line_ids.len() {
            return Err(anyhow::anyhow!("Some line items not found"));
        }

        // 2. 创建清账凭证并添加明细
        let mut clearing_doc = ClearingDocument::new(
            command.company_code.clone(),
            command.fiscal_year,
            command.clearing_date,
            command.currency.clone(),
            command.created_by,
        );

        for line in lines {
            if line.amount.currency() != command.currency {
                return Err(anyhow::anyhow!("Currency mismatch for line {}", line.line_number));
            }
            // 简单添加，目前领域模型限制暂不强制传 UUID
            clearing_doc.add_item(Uuid::nil(), line.amount.amount());
        }

        // 3. 验证平衡
        if !clearing_doc.validate_balance() {
            return Err(anyhow::anyhow!("Clearing document not balanced (sum != 0)"));
        }

        // 4. 获取清账凭证号
        let clearing_number = self.repository.next_document_number(&command.company_code, command.fiscal_year).await?;
        clearing_doc.clearing_number = format!("CL-{}", clearing_number);

        // 5. 保存并更新状态
        self.repository.save_clearing_document(&clearing_doc).await?;

        Ok(clearing_doc)
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

        if let Some(rate) = cmd.exchange_rate {
            entry.header_mut().exchange_rate = rate;
        }

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

            if let Some(tax_code) = &line_cmd.tax_code {
                line = line.with_tax_code(tax_code);
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

        // 1. 计算税务 (如果已有税码)
        let tax_service = MockTaxService;
        entry.calculate_taxes(&tax_service)?;

        // 2. 处理本位币转换 (如果汇率 > 0)
        let rate_val = entry.header().exchange_rate;
        if rate_val > rust_decimal::Decimal::ZERO {
            let rate = ExchangeRate::new(rate_val)?;
            for line in entry.lines_mut() {
                line.apply_exchange_rate(rate);
            }
        }

        // 3. 过账
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

    /// 冲销凭证
    #[instrument(skip(self))]
    pub async fn reverse_journal_entry(
        &self,
        id: Uuid,
        reversal_reason: &str,
        reversal_date: Option<chrono::NaiveDate>,
        posted_by: Uuid,
    ) -> anyhow::Result<(JournalEntry, JournalEntry)> {
        // 1. 获取原凭证
        let mut original_entry = self.repository.find_by_id(&id).await?
            .ok_or_else(|| anyhow::anyhow!("Original journal entry not found: {}", id))?;

        // 2. 执行领域冲销逻辑，生成冲销凭证
        let reversal_id = JournalEntryId::new();
        let date = reversal_date.unwrap_or_else(|| Utc::now().naive_utc().date());
        
        let mut reversal_entry = original_entry.reverse(
            reversal_id,
            posted_by,
            reversal_reason,
            date
        )?;

        // 3. 为冲销凭证分配凭证号并过账
        let company_code = reversal_entry.header().company_code.clone();
        let fiscal_year = reversal_entry.header().fiscal_period.year();
        let doc_number = self.repository.next_document_number(&company_code, fiscal_year).await?;
        let document_number = DocumentNumber::new(&company_code, fiscal_year, &doc_number)?;
        
        reversal_entry.post(document_number, posted_by)?;

        // 4. 持久化 (原凭证标记为已冲销，保存新的冲销凭证)
        // TODO: 使用数据库事务保证原子性
        self.repository.save(&mut original_entry).await?;
        self.repository.save(&mut reversal_entry).await?;

        info!(
            original_id = %id,
            reversal_id = %reversal_entry.id(),
            "Journal entry reversed"
        );

        Ok((original_entry, reversal_entry))
    }

    /// 暂存凭证
    #[instrument(skip(self))]
    pub async fn park_journal_entry(
        &self,
        command: CreateJournalEntryCommand,
    ) -> anyhow::Result<JournalEntry> {
        // 1. 创建凭证对象 (初始为 Draft)
        let mut entry = self.create_journal_entry(command).await?;
        
        // 2. 转换为暂存状态 (不需要验证平衡)
        entry.park()?;
        
        // 3. 持久化
        self.repository.save(&mut entry).await?;
        
        Ok(entry)
    }
}
