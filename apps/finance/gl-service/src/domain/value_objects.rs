//! Value Objects for GL Service Domain
//!
//! 不可变值对象，封装业务规则和验证逻辑

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// 错误类型
// ============================================================================

#[derive(Debug, Error)]
pub enum ValueError {
    #[error("Invalid document number: {0}")]
    InvalidDocumentNumber(String),
    
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    
    #[error("Invalid account: {0}")]
    InvalidAccount(String),
    
    #[error("Invalid fiscal period: {0}")]
    InvalidFiscalPeriod(String),
    
    #[error("Debit credit imbalance: debit={debit}, credit={credit}")]
    DebitCreditImbalance { debit: Decimal, credit: Decimal },
}

// ============================================================================
// JournalEntryId
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JournalEntryId(Uuid);

impl JournalEntryId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
    
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for JournalEntryId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for JournalEntryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// DocumentNumber - 凭证号 (公司代码 + 年度 + 序号)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentNumber {
    company_code: String,
    fiscal_year: i32,
    number: String,
}

impl DocumentNumber {
    pub fn new(company_code: &str, fiscal_year: i32, number: &str) -> Result<Self, ValueError> {
        if company_code.len() > 4 {
            return Err(ValueError::InvalidDocumentNumber(
                "Company code must be at most 4 characters".into()
            ));
        }
        if number.is_empty() || number.len() > 20 {
            return Err(ValueError::InvalidDocumentNumber(
                "Document number must be 1-20 characters".into()
            ));
        }
        
        Ok(Self {
            company_code: company_code.to_uppercase(),
            fiscal_year,
            number: number.to_string(),
        })
    }
    
    pub fn company_code(&self) -> &str { &self.company_code }
    pub fn fiscal_year(&self) -> i32 { self.fiscal_year }
    pub fn number(&self) -> &str { &self.number }
}

impl fmt::Display for DocumentNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}-{}", self.company_code, self.fiscal_year, self.number)
    }
}

// ============================================================================
// MonetaryAmount - 金额 (含币种和借贷方向)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebitCreditIndicator {
    Debit,  // S - 借方
    Credit, // H - 贷方
}

impl DebitCreditIndicator {
    pub fn as_str(&self) -> &'static str {
        match self {
            DebitCreditIndicator::Debit => "S",
            DebitCreditIndicator::Credit => "H",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "S" | "DEBIT" => Some(DebitCreditIndicator::Debit),
            "H" | "CREDIT" => Some(DebitCreditIndicator::Credit),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonetaryAmount {
    amount: Decimal,
    currency: String,
    dc_indicator: DebitCreditIndicator,
}

impl MonetaryAmount {
    pub fn new(amount: Decimal, currency: &str, dc_indicator: DebitCreditIndicator) -> Result<Self, ValueError> {
        if amount < Decimal::ZERO {
            return Err(ValueError::InvalidAmount("Amount cannot be negative".into()));
        }
        if currency.len() != 3 {
            return Err(ValueError::InvalidAmount("Currency must be 3 characters".into()));
        }
        
        Ok(Self {
            amount,
            currency: currency.to_uppercase(),
            dc_indicator,
        })
    }
    
    pub fn amount(&self) -> Decimal { self.amount }
    pub fn currency(&self) -> &str { &self.currency }
    pub fn dc_indicator(&self) -> DebitCreditIndicator { self.dc_indicator }
    
    /// 返回带符号的金额 (借方为正，贷方为负)
    pub fn signed_amount(&self) -> Decimal {
        match self.dc_indicator {
            DebitCreditIndicator::Debit => self.amount,
            DebitCreditIndicator::Credit => -self.amount,
        }
    }
}

// ============================================================================
// Account - 科目 (总账/客户/供应商/资产)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    GeneralLedger,  // S - 总账科目
    Customer,       // D - 客户
    Vendor,         // K - 供应商
    Asset,          // A - 资产
    Material,       // M - 物料
}

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountType::GeneralLedger => "S",
            AccountType::Customer => "D",
            AccountType::Vendor => "K",
            AccountType::Asset => "A",
            AccountType::Material => "M",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Account {
    account_type: AccountType,
    gl_account: String,
    subledger_account: Option<String>,
}

impl Account {
    pub fn gl_account(account_number: &str) -> Result<Self, ValueError> {
        if account_number.is_empty() || account_number.len() > 10 {
            return Err(ValueError::InvalidAccount("GL account must be 1-10 characters".into()));
        }
        
        Ok(Self {
            account_type: AccountType::GeneralLedger,
            gl_account: account_number.to_string(),
            subledger_account: None,
        })
    }
    
    pub fn customer(gl_account: &str, customer_number: &str) -> Result<Self, ValueError> {
        Ok(Self {
            account_type: AccountType::Customer,
            gl_account: gl_account.to_string(),
            subledger_account: Some(customer_number.to_string()),
        })
    }
    
    pub fn vendor(gl_account: &str, vendor_number: &str) -> Result<Self, ValueError> {
        Ok(Self {
            account_type: AccountType::Vendor,
            gl_account: gl_account.to_string(),
            subledger_account: Some(vendor_number.to_string()),
        })
    }
    
    pub fn account_type(&self) -> &AccountType { &self.account_type }
    pub fn get_gl_account(&self) -> &str { &self.gl_account }
    pub fn subledger_account(&self) -> Option<&str> { self.subledger_account.as_deref() }
}

// ============================================================================
// CostObjects - 成本对象
// ============================================================================

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CostObjects {
    pub cost_center: Option<String>,
    pub profit_center: Option<String>,
    pub internal_order: Option<String>,
    pub wbs_element: Option<String>,
    pub business_area: Option<String>,
    pub functional_area: Option<String>,
    pub segment: Option<String>,
}

impl CostObjects {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_cost_center(mut self, cost_center: &str) -> Self {
        self.cost_center = Some(cost_center.to_string());
        self
    }
    
    pub fn with_profit_center(mut self, profit_center: &str) -> Self {
        self.profit_center = Some(profit_center.to_string());
        self
    }
}

// ============================================================================
// FiscalPeriod - 会计期间
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FiscalPeriod {
    year: i32,
    period: i32,
}

impl FiscalPeriod {
    pub fn new(year: i32, period: i32) -> Result<Self, ValueError> {
        if period < 1 || period > 16 {
            return Err(ValueError::InvalidFiscalPeriod(
                format!("Period must be 1-16, got {}", period)
            ));
        }
        
        Ok(Self { year, period })
    }
    
    pub fn year(&self) -> i32 { self.year }
    pub fn period(&self) -> i32 { self.period }
    
    /// 是否为特殊期间 (13-16)
    pub fn is_special_period(&self) -> bool {
        self.period > 12
    }
}

impl fmt::Display for FiscalPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{:02}", self.year, self.period)
    }
}
// ============================================================================
// ReversalType - 冲销类型
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReversalType {
    Full,            // 完全冲销 (借贷反转)
    Partial,         // 部分冲销
    NegativePosting, // 负数冲销 (红字冲销)
}

impl ReversalType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReversalType::Full => "FULL",
            ReversalType::Partial => "PARTIAL",
            ReversalType::NegativePosting => "NEGATIVE",
        }
    }
}

// ============================================================================
// ExchangeRate - 汇率
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExchangeRate(Decimal);

impl ExchangeRate {
    pub fn new(rate: Decimal) -> Result<Self, ValueError> {
        if rate <= Decimal::ZERO {
            return Err(ValueError::InvalidAmount("Exchange rate must be positive".into()));
        }
        Ok(Self(rate))
    }

    pub fn value(&self) -> Decimal {
        self.0
    }

    /// 将外币金额转换为本币金额
    pub fn convert_to_local(&self, amount: Decimal) -> Decimal {
        (amount * self.0).round_dp(2)
    }
}

// ============================================================================
// TaxType - 税务类型
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaxType {
    Input,  // VST - 进项税
    Output, // MWS - 销项税
}

impl TaxType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaxType::Input => "VST",
            TaxType::Output => "MWS",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "VST" | "INPUT" => Some(TaxType::Input),
            "MWS" | "OUTPUT" => Some(TaxType::Output),
            _ => None,
        }
    }
}
