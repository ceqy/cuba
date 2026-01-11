use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingCondition {
    pub condition_id: Uuid,
    pub condition_type: String,
    pub material: Option<String>,
    pub customer: Option<String>,
    pub sales_org: String,
    pub amount: Decimal,
    pub currency: String,
    pub valid_from: Option<NaiveDate>,
    pub valid_to: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingResult {
    pub item_id: String,
    pub net_price: Decimal,
    pub tax_amount: Decimal,
    pub gross_price: Decimal,
    pub conditions: Vec<AppliedCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedCondition {
    pub condition_type: String,
    pub value: Decimal,
    pub currency: String,
    pub description: String,
}
