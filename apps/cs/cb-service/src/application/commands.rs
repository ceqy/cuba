use serde::Deserialize;
use chrono::NaiveDate;
use rust_decimal::Decimal;

#[derive(Debug, Deserialize)]
pub struct CreateBillingPlanCommand {
    pub contract_number: String,
    pub customer_id: String,
    pub validity_start: NaiveDate,
    pub validity_end: NaiveDate,
    pub items: Vec<BillingItem>,
}

#[derive(Debug, Deserialize)]
pub struct BillingItem {
    pub planned_date: NaiveDate,
    pub amount: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct RunBillingCommand {
    pub until_date: NaiveDate,
}
