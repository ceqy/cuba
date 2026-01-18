use std::sync::Arc;
use crate::application::commands::{CreateJournalEntryCommand, PostJournalEntryCommand, ReverseJournalEntryCommand, ParkJournalEntryCommand, UpdateJournalEntryCommand};
use crate::application::queries::{GetJournalEntryQuery, ListJournalEntriesQuery, ListSpecialGlEntriesQuery, ListBusinessPartnerSpecialGlQuery};
use crate::domain::aggregates::journal_entry::{JournalEntry, LineItem, DebitCredit, PostingStatus};
use crate::domain::repositories::JournalRepository;
use crate::domain::services::AccountValidationService;
use uuid::Uuid;

pub struct CreateJournalEntryHandler<R> {
    repository: Arc<R>,
    account_validation: Option<Arc<AccountValidationService>>,
}

impl<R: JournalRepository> CreateJournalEntryHandler<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self {
            repository,
            account_validation: None,
        }
    }

    pub fn with_account_validation(mut self, validation: Arc<AccountValidationService>) -> Self {
        self.account_validation = Some(validation);
        self
    }

    /// 验证特殊总账业务规则
    fn validate_special_gl_rules(lines: &[LineItem]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::domain::aggregates::journal_entry::SpecialGlType;

        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1;

            match line.special_gl_indicator {
                // 预付款业务规则
                SpecialGlType::DownPayment => {
                    // TODO: 预付款必须关联业务伙伴（供应商）
                    // 当前 LineItem 结构中没有 business_partner 字段
                    // 这个验证需要在添加 business_partner 字段后实现
                    tracing::debug!("Line {}: Down payment detected", line_num);
                }

                // 预收款业务规则
                SpecialGlType::AdvancePayment => {
                    // TODO: 预收款必须关联业务伙伴（客户）
                    // 当前 LineItem 结构中没有 business_partner 字段
                    // 这个验证需要在添加 business_partner 字段后实现
                    tracing::debug!("Line {}: Advance payment detected", line_num);
                }

                // 票据业务规则
                SpecialGlType::BillOfExchange | SpecialGlType::BillDiscount => {
                    // TODO: 票据必须有到期日信息
                    // 当前 LineItem 结构中没有 maturity_date 字段
                    // 这个验证需要在添加 maturity_date 字段后实现
                    tracing::debug!("Line {}: Bill of exchange detected", line_num);
                }

                SpecialGlType::Normal => {
                    // 普通业务，无需特殊验证
                }
            }
        }

        Ok(())
    }

    pub async fn handle(&self, cmd: CreateJournalEntryCommand) -> Result<JournalEntry, Box<dyn std::error::Error + Send + Sync>> {
        // Validate accounts if COA service is available
        if let Some(validator) = &self.account_validation {
            let account_codes: Vec<String> = cmd.lines.iter()
                .map(|l| l.account_id.clone())
                .collect();

            tracing::info!("Validating {} accounts via COA service", account_codes.len());

            match validator.validate_journal_entry_accounts(
                account_codes,
                &cmd.company_code,
                cmd.posting_date,
            ).await {
                Ok(_) => {
                    tracing::info!("All accounts validated successfully");
                }
                Err(e) => {
                    tracing::error!("Account validation failed: {}", e);
                    return Err(format!("科目验证失败: {}", e).into());
                }
            }
        } else {
            tracing::warn!("COA service not available, skipping account validation");
        }

        let lines: Vec<LineItem> = cmd.lines.into_iter().enumerate().map(|(i, l)| -> Result<LineItem, Box<dyn std::error::Error + Send + Sync>> {
            // 解析特殊总账标识
            let special_gl_indicator = if let Some(ref code) = l.special_gl_indicator {
                crate::domain::aggregates::journal_entry::SpecialGlType::from_sap_code(code)
            } else {
                crate::domain::aggregates::journal_entry::SpecialGlType::Normal
            };

            // 解析并行会计字段
            let ledger = l.ledger.unwrap_or_else(|| "0L".to_string());
            let ledger_type = if let Some(lt) = l.ledger_type {
                crate::domain::aggregates::journal_entry::LedgerType::from(lt)
            } else {
                crate::domain::aggregates::journal_entry::LedgerType::Leading
            };

            Ok(LineItem {
                id: Uuid::new_v4(),
                line_number: (i + 1) as i32,
                account_id: l.account_id,
                debit_credit: match l.debit_credit.as_str() {
                    "S" | "D" => DebitCredit::Debit,
                    "H" | "C" => DebitCredit::Credit,
                    _ => return Err(format!("Invalid debit/credit indicator: {}", l.debit_credit).into()),
                },
                amount: l.amount,
                local_amount: l.amount,
                cost_center: l.cost_center,
                profit_center: l.profit_center,
                text: l.text,
                special_gl_indicator,
                ledger,
                ledger_type,
                ledger_amount: l.ledger_amount,
            })
        }).collect::<Result<Vec<_>, _>>()?;

        // 验证特殊总账业务规则
        Self::validate_special_gl_rules(&lines)?;

        // Create aggregate
        let mut entry = JournalEntry::new(
            cmd.company_code,
            cmd.fiscal_year,
            cmd.posting_date,
            cmd.document_date,
            cmd.currency,
            cmd.reference,
            lines,
            None,
        )?;

        if cmd.post_immediately {
            let doc_num = format!("DOC-{}-{}", entry.fiscal_year, Uuid::new_v4().simple().to_string().chars().take(8).collect::<String>());
            entry.post(doc_num)?;
        }

        self.repository.save(&entry).await?;

        Ok(entry)
    }
}

pub struct PostJournalEntryHandler<R> {
    repository: Arc<R>,
}

impl<R: JournalRepository> PostJournalEntryHandler<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn handle(&self, cmd: PostJournalEntryCommand) -> Result<JournalEntry, Box<dyn std::error::Error + Send + Sync>> {
        let mut entry = self.repository.find_by_id(&cmd.id).await?
            .ok_or("Journal entry not found")?;

        if entry.status == PostingStatus::Posted {
            return Ok(entry);
        }

        let doc_num = format!("DOC-{}-{}", entry.fiscal_year, Uuid::new_v4().simple().to_string().chars().take(8).collect::<String>());
        entry.post(doc_num)?;

        self.repository.save(&entry).await?;

        Ok(entry)
    }
}

pub struct GetJournalEntryHandler<R> {
    repository: Arc<R>,
}

impl<R: JournalRepository> GetJournalEntryHandler<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn handle(&self, query: GetJournalEntryQuery) -> Result<Option<JournalEntry>, Box<dyn std::error::Error + Send + Sync>> {
        self.repository.find_by_id(&query.id).await
    }
}

pub struct ListJournalEntriesHandler<R> {
    repository: Arc<R>,
}

/// Result with pagination info
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total_items: i64,
    pub page: u64,
    pub page_size: u64,
}

impl<R: JournalRepository> ListJournalEntriesHandler<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn handle(&self, query: ListJournalEntriesQuery) -> Result<PaginatedResult<JournalEntry>, Box<dyn std::error::Error + Send + Sync>> {
        let status = query.status.as_deref();
        let items = self.repository.search(&query.company_code, status, query.page, query.page_size).await?;
        let total_items = self.repository.count(&query.company_code, status).await?;

        Ok(PaginatedResult {
            items,
            total_items,
            page: query.page,
            page_size: query.page_size,
        })
    }
}

pub struct ReverseJournalEntryHandler<R> {
    repository: Arc<R>,
}

impl<R: JournalRepository> ReverseJournalEntryHandler<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn handle(&self, cmd: ReverseJournalEntryCommand) -> Result<JournalEntry, Box<dyn std::error::Error + Send + Sync>> {
        let mut original_entry = self.repository.find_by_id(&cmd.id).await?
            .ok_or("Journal entry not found")?;

        if original_entry.status != PostingStatus::Posted {
            return Err("只能冲销已过账的凭证".into());
        }

        // 使用提供的日期或当前日期作为冲销日期
        let reversal_date = cmd.posting_date.unwrap_or_else(|| chrono::Utc::now().naive_utc().date());

        // 创建冲销凭证
        let reversal_entry = original_entry.create_reversal_entry(reversal_date)?;

        // 保存冲销凭证
        self.repository.save(&reversal_entry).await?;

        // 标记原凭证为已冲销
        original_entry.mark_as_reversed();
        self.repository.save(&original_entry).await?;

        Ok(reversal_entry)
    }
}

pub struct DeleteJournalEntryHandler<R> {
    repository: Arc<R>,
}

impl<R: JournalRepository> DeleteJournalEntryHandler<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn handle(&self, id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check if entry exists and is in draft status
        let entry = self.repository.find_by_id(&id).await?
            .ok_or("Journal entry not found")?;

        if entry.status != PostingStatus::Draft {
            return Err("只能删除草稿状态的凭证".into());
        }

        self.repository.delete(&id).await?;
        Ok(())
    }
}

pub struct ParkJournalEntryHandler<R> {
    repository: Arc<R>,
}

impl<R: JournalRepository> ParkJournalEntryHandler<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn handle(&self, cmd: ParkJournalEntryCommand) -> Result<JournalEntry, Box<dyn std::error::Error + Send + Sync>> {
        let mut entry = self.repository.find_by_id(&cmd.id).await?
            .ok_or("Journal entry not found")?;

        entry.park()?;
        self.repository.save(&entry).await?;

        Ok(entry)
    }
}

pub struct UpdateJournalEntryHandler<R> {
    repository: Arc<R>,
}

impl<R: JournalRepository> UpdateJournalEntryHandler<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn handle(&self, cmd: UpdateJournalEntryCommand) -> Result<JournalEntry, Box<dyn std::error::Error + Send + Sync>> {
        let mut entry = self.repository.find_by_id(&cmd.id).await?
            .ok_or("Journal entry not found")?;

        // Convert LineItemDTO to LineItem if provided
        let lines = if let Some(line_dtos) = cmd.lines {
            let converted_lines: Vec<LineItem> = line_dtos.into_iter().enumerate().map(|(i, l)| -> Result<LineItem, Box<dyn std::error::Error + Send + Sync>> {
                // 解析特殊总账标识
                let special_gl_indicator = if let Some(ref code) = l.special_gl_indicator {
                    crate::domain::aggregates::journal_entry::SpecialGlType::from_sap_code(code)
                } else {
                    crate::domain::aggregates::journal_entry::SpecialGlType::Normal
                };

                // 解析并行会计字段
                let ledger = l.ledger.unwrap_or_else(|| "0L".to_string());
                let ledger_type = if let Some(lt) = l.ledger_type {
                    crate::domain::aggregates::journal_entry::LedgerType::from(lt)
                } else {
                    crate::domain::aggregates::journal_entry::LedgerType::Leading
                };

                Ok(LineItem {
                    id: Uuid::new_v4(),
                    line_number: (i + 1) as i32,
                    account_id: l.account_id,
                    debit_credit: match l.debit_credit.as_str() {
                        "S" | "D" => DebitCredit::Debit,
                        "H" | "C" => DebitCredit::Credit,
                        _ => return Err(format!("Invalid debit/credit indicator: {}", l.debit_credit).into()),
                    },
                    amount: l.amount,
                    local_amount: l.amount,
                    cost_center: l.cost_center,
                    profit_center: l.profit_center,
                    text: l.text,
                    special_gl_indicator,
                    ledger,
                    ledger_type,
                    ledger_amount: l.ledger_amount,
                })
            }).collect::<Result<Vec<_>, _>>()?;
            Some(converted_lines)
        } else {
            None
        };

        entry.update(cmd.posting_date, cmd.document_date, cmd.reference, lines)?;
        self.repository.save(&entry).await?;

        Ok(entry)
    }
}

/// 按特殊总账类型查询处理器
pub struct ListSpecialGlEntriesHandler<R> {
    repository: Arc<R>,
}

impl<R: JournalRepository> ListSpecialGlEntriesHandler<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn handle(&self, query: ListSpecialGlEntriesQuery) -> Result<PaginatedResult<JournalEntry>, Box<dyn std::error::Error + Send + Sync>> {
        use crate::domain::aggregates::journal_entry::SpecialGlType;

        // 验证特殊总账类型
        let gl_type = SpecialGlType::from_sap_code(&query.special_gl_type);
        if !gl_type.is_special() {
            return Err(format!("无效的特殊总账类型: {}. 有效值: A (票据), F (预付款), V (预收款), W (票据贴现)", query.special_gl_type).into());
        }

        // 获取所有凭证
        let status = query.status.as_deref();
        let all_items = self.repository.search(&query.company_code, status, 1, 10000).await?;

        // 过滤包含指定特殊总账类型的凭证
        let filtered_items: Vec<JournalEntry> = all_items.into_iter()
            .filter(|entry| {
                entry.lines.iter().any(|line| {
                    line.special_gl_indicator == gl_type
                })
            })
            .collect();

        // 分页
        let total_items = filtered_items.len() as i64;
        let offset = ((query.page.max(1) - 1) * query.page_size) as usize;
        let items: Vec<JournalEntry> = filtered_items.into_iter()
            .skip(offset)
            .take(query.page_size as usize)
            .collect();

        Ok(PaginatedResult {
            items,
            total_items,
            page: query.page,
            page_size: query.page_size,
        })
    }
}

/// 按业务伙伴和特殊总账类型查询处理器
pub struct ListBusinessPartnerSpecialGlHandler<R> {
    repository: Arc<R>,
}

impl<R: JournalRepository> ListBusinessPartnerSpecialGlHandler<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn handle(&self, query: ListBusinessPartnerSpecialGlQuery) -> Result<PaginatedResult<JournalEntry>, Box<dyn std::error::Error + Send + Sync>> {
        use crate::domain::aggregates::journal_entry::SpecialGlType;

        // 验证特殊总账类型（如果提供）
        let gl_type_filter = if let Some(ref gl_code) = query.special_gl_type {
            let gl_type = SpecialGlType::from_sap_code(gl_code);
            if !gl_type.is_special() {
                return Err(format!("无效的特殊总账类型: {}. 有效值: A (票据), F (预付款), V (预收款), W (票据贴现)", gl_code).into());
            }
            Some(gl_type)
        } else {
            None
        };

        // 获取所有凭证
        let status = query.status.as_deref();
        let all_items = self.repository.search(&query.company_code, status, 1, 10000).await?;

        // 过滤包含指定业务伙伴和特殊总账类型的凭证
        let filtered_items: Vec<JournalEntry> = all_items.into_iter()
            .filter(|entry| {
                entry.lines.iter().any(|line| {
                    // TODO: 当前 LineItem 没有 business_partner 字段
                    // 这个过滤逻辑需要在添加 business_partner 字段后完善
                    // 暂时只过滤特殊总账类型
                    if let Some(gl_type) = gl_type_filter {
                        line.special_gl_indicator == gl_type
                    } else {
                        line.special_gl_indicator.is_special()
                    }
                })
            })
            .collect();

        // 分页
        let total_items = filtered_items.len() as i64;
        let offset = ((query.page.max(1) - 1) * query.page_size) as usize;
        let items: Vec<JournalEntry> = filtered_items.into_iter()
            .skip(offset)
            .take(query.page_size as usize)
            .collect();

        Ok(PaginatedResult {
            items,
            total_items,
            page: query.page,
            page_size: query.page_size,
        })
    }
}
