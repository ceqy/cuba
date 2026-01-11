use serde::Deserialize;
use rust_decimal::Decimal;
use chrono::NaiveDate;

#[derive(Debug, Deserialize)]
pub struct CalculatePriceCommand {
    pub sales_org: String,
    pub customer: Option<String>,
    pub pricing_date: NaiveDate,
    pub items: Vec<PricingItemInput>,
}

#[derive(Debug, Deserialize)]
pub struct PricingItemInput {
    pub item_id: String,
    pub material: String,
    pub quantity: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct UpdateConditionCommand {
    pub condition_type: String,
    pub material: Option<String>,
    pub customer: Option<String>,
    pub sales_org: String,
    pub amount: Decimal,
    pub currency: String,
    pub valid_from: Option<NaiveDate>,
    pub valid_to: Option<NaiveDate>,
}
