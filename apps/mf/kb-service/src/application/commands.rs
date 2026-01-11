use serde::Deserialize;
use rust_decimal::Decimal;

#[derive(Debug, Deserialize)]
pub struct CreateCycleCommand {
    pub material: String,
    pub plant: String,
    pub supply_area: Option<String>,
    pub number_of_kanbans: i32,
    pub qty_per_kanban: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct ChangeStatusCommand {
    pub container_code: String,
    pub new_status: String,
}
