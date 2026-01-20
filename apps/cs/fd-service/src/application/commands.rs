use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateOrderCommand {
    pub customer_id: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct AssignTechnicianCommand {
    pub order_number: String,
    pub technician_id: String,
    pub scheduled_time: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusCommand {
    pub order_number: String,
    pub new_status: String,
}
