use serde::Deserialize;
use rust_decimal::Decimal;
use chrono::NaiveDate;

#[derive(Debug, Deserialize)]
pub struct CreateProductionOrderCommand {
    pub material: String,
    pub plant: String,
    pub quantity: Decimal,
    pub unit: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmOperationCommand {
    pub order_number: String,
    pub operation_number: String,
    pub yield_quantity: Decimal,
    pub scrap_quantity: Decimal,
    pub final_confirmation: bool,
    pub posting_date: NaiveDate,
}
