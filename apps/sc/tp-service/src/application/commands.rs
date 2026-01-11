use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateShipmentCommand {
    pub delivery_numbers: Vec<String>,
    pub planning_point: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusCommand {
    pub shipment_number: String,
    pub new_status: String,
}
