use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{NaiveDate, DateTime, Utc};
use uuid::Uuid;
use cuba_core::domain::AggregateRoot;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostingStatus {
    Draft,
    Parked,
    Posted,
    Reversed,
}

impl Default for PostingStatus {
    fn default() -> Self {
        Self::Draft
    }
}

impl  ToString for PostingStatus {
    fn to_string(&self) -> String {
        match self {
            PostingStatus::Draft => "DRAFT".to_string(),
            PostingStatus::Parked => "PARKED".to_string(),
            PostingStatus::Posted => "POSTED".to_string(),
            PostingStatus::Reversed => "REVERSED".to_string(),
        }
    }
}

impl std::str::FromStr for PostingStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DRAFT" => Ok(PostingStatus::Draft),
            "PARKED" => Ok(PostingStatus::Parked),
            "POSTED" => Ok(PostingStatus::Posted),
            "REVERSED" => Ok(PostingStatus::Reversed),
            _ => Err(format!("Invalid status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebitCredit {
    Debit,
    Credit,
}

impl DebitCredit {
    pub fn as_char(&self) -> char {
        match self {
            DebitCredit::Debit => 'D',
            DebitCredit::Credit => 'C',
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'D' => Some(DebitCredit::Debit),
            'C' => Some(DebitCredit::Credit),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItem {
    pub id: Uuid,
    pub line_number: i32,
    pub account_id: String,
    pub debit_credit: DebitCredit,
    pub amount: Decimal,
    pub local_amount: Decimal,
    pub cost_center: Option<String>,
    pub profit_center: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: Uuid,
    pub document_number: Option<String>,
    pub company_code: String,
    pub fiscal_year: i32,
    pub posting_date: NaiveDate,
    pub document_date: NaiveDate,
    pub currency: String,
    pub reference: Option<String>,
    pub status: PostingStatus,
    pub lines: Vec<LineItem>,
    pub created_at: DateTime<Utc>,
    pub posted_at: Option<DateTime<Utc>>,
    pub tenant_id: Option<String>,
}

#[derive(Error, Debug)]
pub enum JournalEntryError {
    #[error("Debits ({debit}) must equal Credits ({credit})")]
    BalanceError { debit: Decimal, credit: Decimal },
    #[error("Journal entry is already posted")]
    AlreadyPosted,
    #[error("Journal entry is not posted")]
    NotPosted,
    #[error("Empty lines")]
    EmptyLines,
}

impl JournalEntry {
    pub fn new(
        company_code: String,
        fiscal_year: i32,
        posting_date: NaiveDate,
        document_date: NaiveDate,
        currency: String,
        reference: Option<String>,
        lines: Vec<LineItem>,
        tenant_id: Option<String>,
    ) -> Result<Self, JournalEntryError> {
        if lines.is_empty() {
             return Err(JournalEntryError::EmptyLines);
        }

        let entry = Self {
            id: Uuid::new_v4(),
            document_number: None,
            company_code,
            fiscal_year,
            posting_date,
            document_date,
            currency,
            reference,
            status: PostingStatus::Draft,
            lines,
            created_at: Utc::now(),
            posted_at: None,
            tenant_id,
        };
        
        Ok(entry)
    }

    pub fn validate_balance(&self) -> Result<(), JournalEntryError> {
        let mut debit_sum = Decimal::ZERO;
        let mut credit_sum = Decimal::ZERO;

        for line in &self.lines {
            match line.debit_credit {
                DebitCredit::Debit => debit_sum += line.amount,
                DebitCredit::Credit => credit_sum += line.amount,
            }
        }

        if debit_sum != credit_sum {
            return Err(JournalEntryError::BalanceError { debit: debit_sum, credit: credit_sum });
        }

        Ok(())
    }

    pub fn post(&mut self, document_number: String) -> Result<(), JournalEntryError> {
        if self.status == PostingStatus::Posted {
            return Err(JournalEntryError::AlreadyPosted);
        }

        self.validate_balance()?;

        self.status = PostingStatus::Posted;
        self.document_number = Some(document_number);
        self.posted_at = Some(Utc::now());

        Ok(())
    }

    /// 创建冲销凭证
    pub fn create_reversal_entry(&self, reversal_date: NaiveDate) -> Result<JournalEntry, JournalEntryError> {
        if self.status != PostingStatus::Posted {
            return Err(JournalEntryError::NotPosted);
        }

        // 反转所有行项目的借贷方向
        let reversed_lines: Vec<LineItem> = self.lines.iter().enumerate().map(|(i, line)| LineItem {
            id: Uuid::new_v4(),
            line_number: (i + 1) as i32,
            account_id: line.account_id.clone(),
            debit_credit: match line.debit_credit {
                DebitCredit::Debit => DebitCredit::Credit,
                DebitCredit::Credit => DebitCredit::Debit,
            },
            amount: line.amount,
            local_amount: line.local_amount,
            cost_center: line.cost_center.clone(),
            profit_center: line.profit_center.clone(),
            text: Some(format!("冲销 {}", self.document_number.as_ref().unwrap_or(&"".to_string()))),
        }).collect();

        let mut reversal_entry = JournalEntry::new(
            self.company_code.clone(),
            self.fiscal_year,
            reversal_date,
            reversal_date,
            self.currency.clone(),
            Some(format!("冲销 {}", self.document_number.as_ref().unwrap_or(&"".to_string()))),
            reversed_lines,
            self.tenant_id.clone(),
        )?;

        // 自动过账冲销凭证
        let reversal_doc_num = format!("REV-{}", self.document_number.as_ref().unwrap_or(&Uuid::new_v4().simple().to_string()));
        reversal_entry.post(reversal_doc_num)?;

        Ok(reversal_entry)
    }

    /// 标记原凭证为已冲销
    pub fn mark_as_reversed(&mut self) {
        self.status = PostingStatus::Reversed;
    }

    /// 暂存凭证 (Park)
    pub fn park(&mut self) -> Result<(), JournalEntryError> {
        if self.status == PostingStatus::Posted {
            return Err(JournalEntryError::AlreadyPosted);
        }

        // 验证借贷平衡
        self.validate_balance()?;

        self.status = PostingStatus::Parked;
        Ok(())
    }

    /// 更新凭证 (仅限 Draft 或 Parked 状态)
    pub fn update(
        &mut self,
        posting_date: Option<NaiveDate>,
        document_date: Option<NaiveDate>,
        reference: Option<String>,
        lines: Option<Vec<LineItem>>,
    ) -> Result<(), JournalEntryError> {
        if self.status == PostingStatus::Posted || self.status == PostingStatus::Reversed {
            return Err(JournalEntryError::AlreadyPosted);
        }

        if let Some(pd) = posting_date {
            self.posting_date = pd;
        }
        if let Some(dd) = document_date {
            self.document_date = dd;
        }
        if let Some(r) = reference {
            self.reference = Some(r);
        }
        if let Some(l) = lines {
            if l.is_empty() {
                return Err(JournalEntryError::EmptyLines);
            }
            self.lines = l;
        }

        Ok(())
    }
}

// Implement Entity trait
impl cuba_core::domain::Entity for JournalEntry {
    type Id = Uuid;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

// Implement AggregateRoot marker trait
impl AggregateRoot for JournalEntry {}
