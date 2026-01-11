use serde::Deserialize;
use rust_decimal::Decimal;

#[derive(Debug, Deserialize)]
pub struct CreateNotificationCommand {
    pub notification_type: String,
    pub description: String,
    pub equipment_number: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderCommand {
    pub order_type: String,
    pub description: String,
    pub equipment_number: String,
    pub maintenance_plant: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmOperationCommand {
    pub order_number: String,
    pub operation_number: String,
    pub actual_duration: Decimal,
}
