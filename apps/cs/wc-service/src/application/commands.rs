use serde::Deserialize;
use rust_decimal::Decimal;
use chrono::NaiveDate;

#[derive(Debug, Deserialize)]
pub struct CreateClaimCommand {
    pub customer_id: String,
    pub product_id: String,
    pub failure_date: NaiveDate,
    pub failure_description: Option<String>,
    pub claimed_amount: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct AdjudicateClaimCommand {
    pub claim_id: uuid::Uuid,
    pub new_status: String,
    pub approved_amount: Option<Decimal>,
    pub notes: Option<String>,
}
