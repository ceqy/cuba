//! AR/AP Service - Value Objects
//!
//! 值对象定义（不可变、无标识）

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ============================================================================
// Money - 金额（含币种）
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    pub amount: Decimal,
    pub currency: Currency,
}

impl Money {
    pub fn new(amount: Decimal, currency: Currency) -> Self {
        Self { amount, currency }
    }
    
    pub fn zero(currency: Currency) -> Self {
        Self {
            amount: Decimal::ZERO,
            currency,
        }
    }
    
    pub fn is_zero(&self) -> bool {
        self.amount.is_zero()
    }
    
    pub fn is_positive(&self) -> bool {
        self.amount.is_sign_positive()
    }
    
    pub fn is_negative(&self) -> bool {
        self.amount.is_sign_negative()
    }
    
    pub fn abs(&self) -> Self {
        Self {
            amount: self.amount.abs(),
            currency: self.currency,
        }
    }
}

// ============================================================================
// Currency - 货币
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Currency {
    CNY,
    USD,
    EUR,
    JPY,
    GBP,
    Other(CurrencyCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrencyCode([u8; 3]);

impl Currency {
    pub fn from_code(code: &str) -> Self {
        match code {
            "CNY" => Currency::CNY,
            "USD" => Currency::USD,
            "EUR" => Currency::EUR,
            "JPY" => Currency::JPY,
            "GBP" => Currency::GBP,
            _ => {
                let mut bytes = [0u8; 3];
                let code_bytes = code.as_bytes();
                bytes.copy_from_slice(&code_bytes[..3.min(code_bytes.len())]);
                Currency::Other(CurrencyCode(bytes))
            }
        }
    }
    
    pub fn code(&self) -> String {
        match self {
            Currency::CNY => "CNY".to_string(),
            Currency::USD => "USD".to_string(),
            Currency::EUR => "EUR".to_string(),
            Currency::JPY => "JPY".to_string(),
            Currency::GBP => "GBP".to_string(),
            Currency::Other(code) => {
                String::from_utf8_lossy(&code.0).trim_end_matches('\0').to_string()
            }
        }
    }
}

// ============================================================================
// DocumentReference - 凭证引用
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentReference {
    pub company_code: String,
    pub document_number: String,
    pub fiscal_year: i32,
}

impl DocumentReference {
    pub fn new(company_code: String, document_number: String, fiscal_year: i32) -> Self {
        Self {
            company_code,
            document_number,
            fiscal_year,
        }
    }
    
    pub fn to_string(&self) -> String {
        format!("{}-{}-{}", self.company_code, self.fiscal_year, self.document_number)
    }
}

// ============================================================================
// PaymentTerms - 付款条件
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentTerms {
    pub code: String,
    pub description: Option<String>,
    pub net_days: i32,
    pub discount_days_1: Option<i32>,
    pub discount_percent_1: Option<Decimal>,
    pub discount_days_2: Option<i32>,
    pub discount_percent_2: Option<Decimal>,
}

impl PaymentTerms {
    pub fn calculate_due_date(&self, base_date: NaiveDate) -> NaiveDate {
        base_date + chrono::Duration::days(self.net_days as i64)
    }
    
    pub fn calculate_discount(&self, amount: Decimal, payment_date: NaiveDate, invoice_date: NaiveDate) -> Decimal {
        let days_diff = (payment_date - invoice_date).num_days() as i32;
        
        if let (Some(days), Some(percent)) = (self.discount_days_1, self.discount_percent_1) {
            if days_diff <= days {
                return amount * percent / Decimal::from(100);
            }
        }
        
        if let (Some(days), Some(percent)) = (self.discount_days_2, self.discount_percent_2) {
            if days_diff <= days {
                return amount * percent / Decimal::from(100);
            }
        }
        
        Decimal::ZERO
    }
}

// ============================================================================
// CreditStatus - 信用状态
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreditStatus {
    Ok,
    Warning,
    Exceeded,
    NoLimit,
}

// ============================================================================
// ClearingType - 清账类型
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClearingType {
    Full,       // 全额清账
    Partial,    // 部分清账
    Automatic,  // 自动清账
    Net,        // 净额清账
}

impl ClearingType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "FULL" => Some(ClearingType::Full),
            "PARTIAL" => Some(ClearingType::Partial),
            "AUTOMATIC" => Some(ClearingType::Automatic),
            "NET" => Some(ClearingType::Net),
            _ => None,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            ClearingType::Full => "FULL",
            ClearingType::Partial => "PARTIAL",
            ClearingType::Automatic => "AUTOMATIC",
            ClearingType::Net => "NET",
        }
    }
}

// ============================================================================
// PaymentMethod - 付款方式
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentMethod {
    Wire,       // 电汇
    Check,      // 支票
    ACH,        // ACH
    Card,       // 银行卡
}

impl PaymentMethod {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "WIRE" => Some(PaymentMethod::Wire),
            "CHECK" => Some(PaymentMethod::Check),
            "ACH" => Some(PaymentMethod::ACH),
            "CARD" => Some(PaymentMethod::Card),
            _ => None,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentMethod::Wire => "WIRE",
            PaymentMethod::Check => "CHECK",
            PaymentMethod::ACH => "ACH",
            PaymentMethod::Card => "CARD",
        }
    }
}
