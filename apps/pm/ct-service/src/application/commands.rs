use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateContractCommand {
    pub company_code: String,
    pub supplier: String,
    pub purchasing_org: String,
    pub validity_start: Option<NaiveDate>,
    pub validity_end: Option<NaiveDate>,
    pub target_value: Option<Decimal>,
    pub items: Vec<ContractItemInput>,
}

#[derive(Debug, Deserialize)]
pub struct ContractItemInput {
    pub item_number: i32,
    pub material: Option<String>,
    pub short_text: Option<String>,
    pub target_quantity: Option<Decimal>,
    pub net_price: Option<Decimal>,
    pub plant: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApproveContractCommand {
    pub contract_number: String,
    pub approved: bool,
}
