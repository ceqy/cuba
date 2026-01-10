use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonetaryValue {
    pub currency_code: String,
    pub amount: Decimal,
}

impl MonetaryValue {
    pub fn new(amount: Decimal, currency_code: impl Into<String>) -> Self {
        Self {
            amount,
            currency_code: currency_code.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantityValue {
    pub unit_code: String,
    pub value: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_by: Option<String>,
}
