//! AR/AP Service - Domain Entities
//!
//! 核心领域实体定义

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// BusinessPartner - 业务伙伴
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartnerType {
    Person,
    Organization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessPartner {
    pub id: Uuid,
    pub partner_id: String,
    pub partner_type: PartnerType,
    pub name_org1: Option<String>,
    pub name_last: Option<String>,
    pub name_first: Option<String>,
    pub search_term: Option<String>,
    pub country: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BusinessPartner {
    pub fn display_name(&self) -> String {
        match self.partner_type {
            PartnerType::Organization => {
                self.name_org1.clone().unwrap_or_default()
            }
            PartnerType::Person => {
                format!(
                    "{} {}",
                    self.name_first.as_deref().unwrap_or(""),
                    self.name_last.as_deref().unwrap_or("")
                ).trim().to_string()
            }
        }
    }
}

// ============================================================================
// Customer - 客户
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: Uuid,
    pub customer_id: String,
    pub partner_id: String,
    pub company_code: String,
    pub reconciliation_account: Option<String>,
    pub payment_terms: Option<String>,
    pub credit_limit: Option<Decimal>,
    pub credit_currency: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Supplier - 供应商
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Supplier {
    pub id: Uuid,
    pub supplier_id: String,
    pub partner_id: String,
    pub company_code: String,
    pub reconciliation_account: Option<String>,
    pub payment_terms: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// OpenItem - 未清项
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    Customer,
    Supplier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenItem {
    pub id: Uuid,
    pub company_code: String,
    pub document_number: String,
    pub fiscal_year: i32,
    pub line_item: i32,
    pub account_type: AccountType,
    pub partner_id: String,
    pub posting_date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub amount: Decimal,
    pub currency: String,
    pub open_amount: Decimal,
    pub clearing_date: Option<NaiveDate>,
    pub clearing_doc: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl OpenItem {
    pub fn is_overdue(&self) -> bool {
        if let Some(due_date) = self.due_date {
            due_date < chrono::Utc::now().date_naive() && self.clearing_date.is_none()
        } else {
            false
        }
    }
    
    pub fn is_cleared(&self) -> bool {
        self.clearing_date.is_some()
    }
}

// ============================================================================
// AccountBalance - 账户余额（值对象）
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    pub partner_id: String,
    pub company_code: String,
    pub currency: String,
    pub total_debit: Decimal,
    pub total_credit: Decimal,
    pub balance: Decimal,
    pub open_items_count: i64,
}
