use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{NaiveDate, DateTime, Utc};
use uuid::Uuid;
use cuba_core::domain::AggregateRoot;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostingStatus {
    Draft,
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
