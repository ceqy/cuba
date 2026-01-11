use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationRun {
    pub run_id: Uuid,
    pub controlling_area: String,
    pub fiscal_year: i32,
    pub fiscal_period: i32,
    pub allocation_cycle: String,
    pub allocation_type: String,
    pub test_run: bool,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub senders: Vec<AllocationSender>,
    pub receivers: Vec<AllocationReceiver>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationSender {
    pub sender_id: Uuid,
    pub run_id: Uuid,
    pub sender_object: String,
    pub sent_amount: Decimal,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationReceiver {
    pub receiver_id: Uuid,
    pub run_id: Uuid,
    pub receiver_object: String,
    pub received_amount: Decimal,
    pub currency: String,
}
