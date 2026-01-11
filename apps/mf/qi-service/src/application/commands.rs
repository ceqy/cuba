use serde::Deserialize;
use rust_decimal::Decimal;

#[derive(Debug, Deserialize)]
pub struct CreateInspectionLotCommand {
    pub material: String,
    pub plant: String,
    pub quantity: Decimal,
    pub origin: i32, 
}

#[derive(Debug, Deserialize)]
pub struct RecordResultCommand {
    pub lot_number: String,
    pub characteristic_number: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct MakeUsageDecisionCommand {
    pub lot_number: String,
    pub ud_code: String,
    pub note: String,
}
