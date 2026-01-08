//! Domain Entities for GL Service
//!
//! 聚合根和实体定义

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::events::{DomainEvents, JournalEntryCreated, JournalEntryPosted, JournalEntryReversed};
use crate::domain::rules::JournalEntryStatus;
use crate::domain::value_objects::{
    Account, CostObjects, DebitCreditIndicator, DocumentNumber, FiscalPeriod, 
    JournalEntryId, MonetaryAmount, ValueError, TaxType, ExchangeRate,
};

// ============================================================================
// JournalEntry - 凭证聚合根
// ============================================================================

#[derive(Serialize, Deserialize)]
pub struct JournalEntry {
    // 标识
    id: JournalEntryId,
    document_number: Option<DocumentNumber>,
    
    // 抬头信息
    header: JournalEntryHeader,
    
    // 行项目
    lines: Vec<JournalEntryLine>,
    
    // 税务行项目
    tax_items: Vec<TaxLineItem>,
    
    // 状态
    status: JournalEntryStatus,
    
    // 乐观锁版本
    version: u64,
    
    // 领域事件
    #[serde(skip)]
    events: DomainEvents,
}

impl JournalEntry {
    /// 从持久化层重建聚合根
    pub fn reconstruct(
        id: JournalEntryId,
        document_number: Option<DocumentNumber>,
        header: JournalEntryHeader,
        lines: Vec<JournalEntryLine>,
        tax_items: Vec<TaxLineItem>,
        status: JournalEntryStatus,
        version: u64,
    ) -> Self {
        Self {
            id,
            document_number,
            header,
            lines,
            tax_items,
            status,
            version,
            events: DomainEvents::new(),
        }
    }

    /// 创建新的凭证草稿
    pub fn create_draft(
        company_code: String,
        document_type: String,
        document_date: NaiveDate,
        posting_date: NaiveDate,
        fiscal_period: FiscalPeriod,
        currency: String,
        created_by: Uuid,
    ) -> Self {
        let id = JournalEntryId::new();
        let now = Utc::now();
        
        let header = JournalEntryHeader {
            company_code: company_code.clone(),
            document_type,
            document_date,
            posting_date,
            fiscal_period,
            currency,
            exchange_rate: Decimal::ONE,
            local_currency: "CNY".to_string(),
            header_text: None,
            reference_document: None,
            ledger: "0L".to_string(),
            created_by,
            created_at: now,
            updated_at: now,
        };
        
        let mut entry = Self {
            id,
            document_number: None,
            header,
            lines: Vec::new(),
            tax_items: Vec::new(),
            status: JournalEntryStatus::Draft,
            version: 1,
            events: DomainEvents::new(),
        };
        
        // 记录创建事件
        entry.events.push(JournalEntryCreated {
            journal_entry_id: *id.as_uuid(),
            company_code,
            document_number: String::new(),
            fiscal_year: fiscal_period.year(),
            created_by,
            occurred_at: now,
        });
        
        entry
    }
    
    /// 添加行项目
    pub fn add_line(&mut self, line: JournalEntryLine) -> Result<(), ValueError> {
        if !self.status.is_editable() {
            return Err(ValueError::InvalidAmount("Cannot modify non-draft entry".into()));
        }
        self.lines.push(line);
        self.header.updated_at = Utc::now();
        Ok(())
    }

    /// 替换所有行项目
    pub fn replace_lines(&mut self, lines: Vec<JournalEntryLine>) -> Result<(), ValueError> {
        if !self.status.is_editable() {
            return Err(ValueError::InvalidAmount("Cannot modify non-draft entry".into()));
        }
        self.lines = lines;
        self.header.updated_at = Utc::now();
        Ok(())
    }

    /// 更新抬头信息
    pub fn update_header(&mut self, text: Option<String>, doc_date: Option<NaiveDate>, post_date: Option<NaiveDate>) -> Result<(), ValueError> {
        if !self.status.is_editable() {
            return Err(ValueError::InvalidAmount("Cannot modify non-draft entry".into()));
        }
        if let Some(t) = text { self.header.header_text = Some(t); }
        if let Some(d) = doc_date { self.header.document_date = d; }
        if let Some(p) = post_date { self.header.posting_date = p; }
        self.header.updated_at = Utc::now();
        Ok(())
    }
    
    /// 验证借贷平衡
    pub fn validate_balance(&self) -> Result<(), ValueError> {
        let amounts: Vec<MonetaryAmount> = self.lines.iter()
            .map(|l| l.amount.clone())
            .collect();
        crate::domain::rules::validate_debit_credit_balance(&amounts)
    }
    
    /// 过账凭证
    pub fn post(&mut self, document_number: DocumentNumber, posted_by: Uuid) -> Result<(), ValueError> {
        // 检查状态转换
        if !self.status.can_transition_to(JournalEntryStatus::Posted) {
            return Err(ValueError::InvalidAmount(
                format!("Cannot post from status {:?}", self.status)
            ));
        }
        
        // 验证借贷平衡
        self.validate_balance()?;
        
        // 更新状态
        self.status = JournalEntryStatus::Posted;
        self.document_number = Some(document_number.clone());
        self.header.updated_at = Utc::now();
        
        // 记录事件
        self.events.push(JournalEntryPosted {
            journal_entry_id: *self.id.as_uuid(),
            company_code: self.header.company_code.clone(),
            document_number: document_number.number().to_string(),
            fiscal_year: self.header.fiscal_period.year(),
            posted_by,
            posting_date: Utc::now(),
            occurred_at: Utc::now(),
        });
        
        Ok(())
    }
    
    // Getters
    pub fn id(&self) -> &JournalEntryId { &self.id }
    pub fn document_number(&self) -> Option<&DocumentNumber> { self.document_number.as_ref() }
    pub fn header(&self) -> &JournalEntryHeader { &self.header }
    pub fn header_mut(&mut self) -> &mut JournalEntryHeader { &mut self.header }
    pub fn lines(&self) -> &[JournalEntryLine] { &self.lines }
    pub fn lines_mut(&mut self) -> &mut [JournalEntryLine] { &mut self.lines }
    pub fn tax_items(&self) -> &[TaxLineItem] { &self.tax_items }
    pub fn status(&self) -> JournalEntryStatus { self.status }
    pub fn version(&self) -> u64 { self.version }
    
    /// 获取并清空领域事件
    pub fn take_events(&mut self) -> Vec<Box<dyn crate::domain::events::DomainEvent>> {
        self.events.take()
    }
    
    /// 计算借方合计
    pub fn total_debit(&self) -> Decimal {
        self.lines.iter()
            .filter(|l| matches!(l.amount.dc_indicator(), DebitCreditIndicator::Debit))
            .map(|l| l.amount.amount())
            .sum()
    }
    
    /// 计算贷方合计
    pub fn total_credit(&self) -> Decimal {
        self.lines.iter()
            .filter(|l| matches!(l.amount.dc_indicator(), DebitCreditIndicator::Credit))
            .map(|l| l.amount.amount())
            .sum()
    }

    /// 检查借贷是否平衡
    pub fn is_balanced(&self) -> bool {
        self.total_debit() == self.total_credit()
    }

    /// 冲销凭证 (生成一个新的冲销凭证)
    pub fn reverse(
        &mut self, 
        reversal_id: JournalEntryId, 
        posted_by: Uuid, 
        reversal_reason: &str,
        reversal_date: NaiveDate
    ) -> Result<Self, ValueError> {
        // 只有已过账的凭证才能冲销
        if self.status != JournalEntryStatus::Posted {
            return Err(ValueError::InvalidAmount(
                format!("Only posted entries can be reversed, current status: {:?}", self.status)
            ));
        }
        
        // 创建反向行项目
        let reversed_lines: Vec<JournalEntryLine> = self.lines.iter()
            .map(|l| l.reverse())
            .collect();
            
        // 创建冲销凭证抬头
        let mut reversal_header = self.header.clone();
        reversal_header.posting_date = reversal_date;
        reversal_header.created_by = posted_by;
        reversal_header.created_at = Utc::now();
        reversal_header.updated_at = Utc::now();
        reversal_header.header_text = Some(format!("Rev: {}", reversal_reason));
        reversal_header.reference_document = self.document_number.as_ref().map(|d| d.to_string());
        
        // 构建冲销凭证聚合根
        let reversal_entry = JournalEntry::reconstruct(
            reversal_id,
            None,
            reversal_header,
            reversed_lines,
            Vec::new(),
            JournalEntryStatus::Draft,
            1,
        );
        
        // 更新原凭证状态
        self.status = JournalEntryStatus::Reversed;
        self.header.updated_at = Utc::now();
        
        // 记录冲销事件
        self.events.push(JournalEntryReversed {
            journal_entry_id: *self.id.as_uuid(),
            reversal_document_id: *reversal_id.as_uuid(),
            reversal_reason: reversal_reason.to_string(),
            reversed_by: posted_by,
            occurred_at: Utc::now(),
        });
        
        Ok(reversal_entry)
    }

    /// 暂存凭证 (不强制校验平衡)
    pub fn park(&mut self) -> Result<(), ValueError> {
        if !self.status.can_transition_to(JournalEntryStatus::Parked) {
            return Err(ValueError::InvalidAmount(
                format!("Cannot park from status {:?}", self.status)
            ));
        }
        self.status = JournalEntryStatus::Parked;
        self.header.updated_at = Utc::now();
        Ok(())
    }

    /// 计算并生成税务行项目 (基于业务行项目的税码)
    pub fn calculate_taxes(&mut self, tax_service: &dyn TaxService) -> Result<(), ValueError> {
        self.tax_items.clear();
        let mut tax_map: std::collections::HashMap<String, (TaxType, Decimal, Decimal, DebitCreditIndicator)> = std::collections::HashMap::new();

        for line in &self.lines {
            if let Some(tax_code) = &line.tax_code {
                if let Some((tax_type, rate)) = tax_service.get_tax_info(tax_code) {
                    let base_amount = line.amount.amount();
                    let tax_amount = (base_amount * rate).round_dp(2);
                    
                    let entry = tax_map.entry(tax_code.clone()).or_insert((
                        tax_type,
                        rate,
                        Decimal::ZERO,
                        line.amount.dc_indicator(),
                    ));
                    
                    entry.2 += tax_amount;
                }
            }
        }

        let mut next_line_no = (self.lines.len() + 1) as i32;
        for (tax_code, (tax_type, rate, total_tax, dc)) in tax_map {
            self.tax_items.push(TaxLineItem {
                line_number: next_line_no,
                tax_code,
                tax_type,
                tax_rate: rate,
                tax_base_amount: Decimal::ZERO, // TODO: Aggregate base amount if needed
                tax_amount: total_tax,
                dc_indicator: dc,
            });
            next_line_no += 1;
        }

        Ok(())
    }
}

// ============================================================================
// JournalEntryHeader - 凭证抬头
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntryHeader {
    pub company_code: String,
    pub document_type: String,
    pub document_date: NaiveDate,
    pub posting_date: NaiveDate,
    pub fiscal_period: FiscalPeriod,
    pub currency: String,
    pub exchange_rate: Decimal,
    pub local_currency: String,
    pub header_text: Option<String>,
    pub reference_document: Option<String>,
    pub ledger: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// JournalEntryLine - 凭证行项目
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntryLine {
    pub line_number: i32,
    pub account: Account,
    pub amount: MonetaryAmount,
    pub amount_local: Option<Decimal>,
    pub cost_objects: CostObjects,
    pub line_text: Option<String>,
    pub assignment: Option<String>,
    pub tax_code: Option<String>,
    pub clearing_status: ClearingStatus,
}

impl JournalEntryLine {
    pub fn new(
        line_number: i32,
        account: Account,
        amount: MonetaryAmount,
    ) -> Self {
        Self {
            line_number,
            account,
            amount,
            amount_local: None,
            cost_objects: CostObjects::default(),
            line_text: None,
            assignment: None,
            tax_code: None,
            clearing_status: ClearingStatus::Open,
        }
    }
    
    pub fn with_cost_objects(mut self, cost_objects: CostObjects) -> Self {
        self.cost_objects = cost_objects;
        self
    }
    
    pub fn with_text(mut self, text: &str) -> Self {
        self.line_text = Some(text.to_string());
        self
    }

    /// 生成反向行项目
    pub fn with_tax_code(mut self, tax_code: &str) -> Self {
        self.tax_code = Some(tax_code.to_string());
        self
    }

    /// 应用汇率计算本币金额
    pub fn apply_exchange_rate(&mut self, rate: ExchangeRate) {
        self.amount_local = Some(rate.convert_to_local(self.amount.amount()));
    }

    pub fn reverse(&self) -> Self {
        let mut reversed = self.clone();
        let reversed_indicator = match self.amount.dc_indicator() {
            DebitCreditIndicator::Debit => DebitCreditIndicator::Credit,
            DebitCreditIndicator::Credit => DebitCreditIndicator::Debit,
        };
        
        // 安全起见直接构造一个新的 MonetaryAmount
        reversed.amount = MonetaryAmount::new(
            self.amount.amount(),
            self.amount.currency(),
            reversed_indicator
        ).expect("Flip D/C should always be valid");
        
        // 如果有本币金额，也需要同步标记？通常本币金额符号随借贷走，但在系统中 amount_local 存储的是绝对值
        // 这里假设绝对值不变，仅靠 dc_indicator 区分
        
        reversed.clearing_status = ClearingStatus::Open;
        reversed
    }
}

// ============================================================================
// ClearingStatus - 清账状态
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ClearingStatus {
    #[default]
    Open,
    PartiallyCleared,
    Cleared,
}

impl ClearingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClearingStatus::Open => "OPEN",
            ClearingStatus::PartiallyCleared => "PARTIALLY_CLEARED",
            ClearingStatus::Cleared => "CLEARED",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "OPEN" => Some(ClearingStatus::Open),
            "PARTIALLY_CLEARED" => Some(ClearingStatus::PartiallyCleared),
            "CLEARED" => Some(ClearingStatus::Cleared),
            _ => None,
        }
    }
}

// ============================================================================
// TaxLineItem - 税务行项目
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxLineItem {
    pub line_number: i32,
    pub tax_code: String,
    pub tax_type: TaxType,
    pub tax_rate: Decimal,
    pub tax_base_amount: Decimal,
    pub tax_amount: Decimal,
    pub dc_indicator: DebitCreditIndicator,
}

// ============================================================================
// TaxService - 税务服务接口
// ============================================================================

pub trait TaxService: Send + Sync {
    /// 获取税率信息
    fn get_tax_info(&self, tax_code: &str) -> Option<(TaxType, Decimal)>;
}
// ============================================================================
// ClearingDocument - 清账凭证
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearingDocument {
    pub id: Uuid,
    pub clearing_number: String,
    pub company_code: String,
    pub fiscal_year: i32,
    pub clearing_date: NaiveDate,
    pub clearing_amount: Decimal,
    pub currency: String,
    pub clearing_type: String,
    pub items: Vec<ClearingItem>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearingItem {
    pub journal_entry_line_id: Uuid,
    pub cleared_amount: Decimal,
}

impl ClearingDocument {
    pub fn new(
        company_code: String,
        fiscal_year: i32,
        clearing_date: NaiveDate,
        currency: String,
        created_by: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            clearing_number: String::new(), // To be assigned
            company_code,
            fiscal_year,
            clearing_date,
            clearing_amount: Decimal::ZERO,
            currency,
            clearing_type: "MANUAL".to_string(),
            items: Vec::new(),
            created_by,
            created_at: Utc::now(),
        }
    }

    pub fn add_item(&mut self, line_id: Uuid, amount: Decimal) {
        self.items.push(ClearingItem {
            journal_entry_line_id: line_id,
            cleared_amount: amount,
        });
        self.clearing_amount += amount;
    }

    /// 验证清账平衡
    pub fn validate_balance(&self) -> bool {
        self.clearing_amount == Decimal::ZERO
    }
}
