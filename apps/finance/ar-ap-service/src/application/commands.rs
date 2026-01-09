//! AR/AP Service - Commands
//!
//! 命令对象 - 表示用户意图

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Partner Commands
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCustomerCommand {
    pub customer_id: String,
    pub partner_id: String,
    pub company_code: String,
    pub reconciliation_account: Option<String>,
    pub payment_terms: Option<String>,
    pub credit_limit: Option<Decimal>,
    pub credit_currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSupplierCommand {
    pub supplier_id: String,
    pub partner_id: String,
    pub company_code: String,
    pub reconciliation_account: Option<String>,
    pub payment_terms: Option<String>,
}

// ============================================================================
// Credit Commands
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformCreditCheckCommand {
    pub customer_id: String,
    pub checked_by: Uuid,
}

// ============================================================================
// Payment Commands
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentProposalCommand {
    pub proposal_id: String,
    pub company_code: String,
    pub proposal_date: NaiveDate,
    pub payment_date: NaiveDate,
    pub payment_method: String,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovePaymentProposalCommand {
    pub proposal_id: Uuid,
    pub approved_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutePaymentRunCommand {
    pub run_id: String,
    pub company_code: String,
    pub run_date: NaiveDate,
    pub payment_method: String,
    pub created_by: Uuid,
}

// ============================================================================
// Clearing Commands
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClearingCommand {
    pub clearing_document: String,
    pub company_code: String,
    pub fiscal_year: i32,
    pub clearing_date: NaiveDate,
    pub cleared_by: Uuid,
    pub clearing_type: String,
    pub total_amount: Decimal,
    pub currency: String,
    pub reference: Option<String>,
}

// ============================================================================
// Dunning Commands
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDunningCommand {
    pub customer_id: String,
    pub dunning_level: i32,
    pub dunning_date: NaiveDate,
    pub total_overdue: Decimal,
    pub currency: String,
    pub contact_method: String,
    pub created_by: Uuid,
}
