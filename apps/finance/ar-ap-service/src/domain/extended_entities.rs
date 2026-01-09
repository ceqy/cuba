//! AR/AP Service - Extended Domain Entities
//!
//! 扩展领域实体（清账、信用、付款等）

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::value_objects::*;

// ============================================================================
// ClearingHistory - 清账历史
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearingHistory {
    pub id: Uuid,
    pub clearing_document: String,
    pub company_code: String,
    pub fiscal_year: i32,
    pub clearing_date: NaiveDate,
    pub cleared_by: Uuid,
    pub clearing_type: ClearingType,
    pub total_amount: Money,
    pub reference: Option<String>,
    pub items: Vec<ClearingItem>,
    pub created_at: DateTime<Utc>,
}

impl ClearingHistory {
    pub fn total_items_count(&self) -> usize {
        self.items.len()
    }
    
    pub fn is_balanced(&self) -> bool {
        let sum: Decimal = self.items.iter().map(|item| item.cleared_amount).sum();
        sum == self.total_amount.amount
    }
}

// ============================================================================
// ClearingItem - 清账行项目
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearingItem {
    pub id: Uuid,
    pub clearing_history_id: Uuid,
    pub open_item_id: Uuid,
    pub cleared_amount: Decimal,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// CreditCheck - 信用检查
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditCheck {
    pub id: Uuid,
    pub customer_id: String,
    pub check_date: NaiveDate,
    pub credit_limit: Option<Money>,
    pub current_exposure: Money,
    pub available_credit: Money,
    pub check_result: CreditCheckResult,
    pub check_reason: Option<String>,
    pub checked_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreditCheckResult {
    Pass,
    Fail,
    Warning,
}

impl CreditCheckResult {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "PASS" => Some(CreditCheckResult::Pass),
            "FAIL" => Some(CreditCheckResult::Fail),
            "WARNING" => Some(CreditCheckResult::Warning),
            _ => None,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            CreditCheckResult::Pass => "PASS",
            CreditCheckResult::Fail => "FAIL",
            CreditCheckResult::Warning => "WARNING",
        }
    }
}

// ============================================================================
// PaymentProposal - 付款建议
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentProposal {
    pub id: Uuid,
    pub proposal_id: String,
    pub company_code: String,
    pub proposal_date: NaiveDate,
    pub payment_date: NaiveDate,
    pub payment_method: PaymentMethod,
    pub total_amount: Money,
    pub status: PaymentProposalStatus,
    pub items: Vec<PaymentProposalItem>,
    pub created_by: Uuid,
    pub approved_by: Option<Uuid>,
    pub executed_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentProposalStatus {
    Draft,
    Approved,
    Executed,
    Cancelled,
}

impl PaymentProposalStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "DRAFT" => Some(PaymentProposalStatus::Draft),
            "APPROVED" => Some(PaymentProposalStatus::Approved),
            "EXECUTED" => Some(PaymentProposalStatus::Executed),
            "CANCELLED" => Some(PaymentProposalStatus::Cancelled),
            _ => None,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentProposalStatus::Draft => "DRAFT",
            PaymentProposalStatus::Approved => "APPROVED",
            PaymentProposalStatus::Executed => "EXECUTED",
            PaymentProposalStatus::Cancelled => "CANCELLED",
        }
    }
}

// ============================================================================
// PaymentProposalItem - 付款建议行项目
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentProposalItem {
    pub id: Uuid,
    pub proposal_id: Uuid,
    pub open_item_id: Uuid,
    pub payment_amount: Decimal,
    pub discount_amount: Decimal,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// DunningHistory - 催款记录
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DunningHistory {
    pub id: Uuid,
    pub customer_id: String,
    pub dunning_level: i32,
    pub dunning_date: NaiveDate,
    pub total_overdue: Money,
    pub contact_method: ContactMethod,
    pub response: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactMethod {
    Email,
    Phone,
    Letter,
}

impl ContactMethod {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "EMAIL" => Some(ContactMethod::Email),
            "PHONE" => Some(ContactMethod::Phone),
            "LETTER" => Some(ContactMethod::Letter),
            _ => None,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            ContactMethod::Email => "EMAIL",
            ContactMethod::Phone => "PHONE",
            ContactMethod::Letter => "LETTER",
        }
    }
}

// ============================================================================
// BankAccount - 银行账户
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccount {
    pub id: Uuid,
    pub account_id: String,
    pub partner_id: String,
    pub bank_key: String,
    pub bank_name: Option<String>,
    pub account_number: String,
    pub iban: Option<String>,
    pub account_holder: Option<String>,
    pub currency: Currency,
    pub is_primary: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// PaymentRun - 付款运行
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRun {
    pub id: Uuid,
    pub run_id: String,
    pub company_code: String,
    pub run_date: NaiveDate,
    pub payment_method: PaymentMethod,
    pub total_payments: i32,
    pub total_amount: Money,
    pub status: PaymentRunStatus,
    pub created_by: Uuid,
    pub executed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentRunStatus {
    Created,
    Processing,
    Completed,
    Failed,
}

impl PaymentRunStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "CREATED" => Some(PaymentRunStatus::Created),
            "PROCESSING" => Some(PaymentRunStatus::Processing),
            "COMPLETED" => Some(PaymentRunStatus::Completed),
            "FAILED" => Some(PaymentRunStatus::Failed),
            _ => None,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentRunStatus::Created => "CREATED",
            PaymentRunStatus::Processing => "PROCESSING",
            PaymentRunStatus::Completed => "COMPLETED",
            PaymentRunStatus::Failed => "FAILED",
        }
    }
}

// ============================================================================
// AdvancePayment - 预付款
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancePayment {
    pub id: Uuid,
    pub advance_id: String,
    pub company_code: String,
    pub partner_id: String,
    pub account_type: super::AccountType,
    pub posting_date: NaiveDate,
    pub amount: Money,
    pub remaining_amount: Money,
    pub gl_account: Option<String>,
    pub reference: Option<String>,
    pub status: AdvancePaymentStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdvancePaymentStatus {
    Active,
    FullyApplied,
    Cancelled,
}

impl AdvancePaymentStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "ACTIVE" => Some(AdvancePaymentStatus::Active),
            "FULLY_APPLIED" => Some(AdvancePaymentStatus::FullyApplied),
            "CANCELLED" => Some(AdvancePaymentStatus::Cancelled),
            _ => None,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            AdvancePaymentStatus::Active => "ACTIVE",
            AdvancePaymentStatus::FullyApplied => "FULLY_APPLIED",
            AdvancePaymentStatus::Cancelled => "CANCELLED",
        }
    }
    
    pub fn is_available(&self) -> bool {
        matches!(self, AdvancePaymentStatus::Active)
    }
}

impl AdvancePayment {
    pub fn can_apply(&self, amount: Decimal) -> bool {
        self.status.is_available() && amount <= self.remaining_amount.amount
    }
    
    pub fn apply(&mut self, amount: Decimal) -> Result<(), String> {
        if !self.can_apply(amount) {
            return Err("Cannot apply amount to advance payment".to_string());
        }
        
        self.remaining_amount.amount -= amount;
        
        if self.remaining_amount.amount.is_zero() {
            self.status = AdvancePaymentStatus::FullyApplied;
        }
        
        Ok(())
    }
}

// ============================================================================
// AgingBucket - 账龄桶（值对象）
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgingBucket {
    pub current: Decimal,
    pub days_1_30: Decimal,
    pub days_31_60: Decimal,
    pub days_61_90: Decimal,
    pub over_90_days: Decimal,
}

impl AgingBucket {
    pub fn total(&self) -> Decimal {
        self.current + self.days_1_30 + self.days_31_60 + self.days_61_90 + self.over_90_days
    }
    
    pub fn overdue_total(&self) -> Decimal {
        self.days_1_30 + self.days_31_60 + self.days_61_90 + self.over_90_days
    }
}
