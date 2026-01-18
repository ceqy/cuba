use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
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
    #[sqlx(skip)]
    pub senders: Vec<AllocationSender>,
    #[sqlx(skip)]
    pub receivers: Vec<AllocationReceiver>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AllocationSender {
    pub sender_id: Uuid,
    pub run_id: Uuid,
    pub sender_object: String,
    pub sent_amount: Decimal,
    pub currency: String,
    pub cost_center: Option<String>,
    pub profit_center: Option<String>,
    pub segment: Option<String>,
    pub internal_order: Option<String>,
    pub wbs_element: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AllocationReceiver {
    pub receiver_id: Uuid,
    pub run_id: Uuid,
    pub receiver_object: String,
    pub received_amount: Decimal,
    pub currency: String,
    pub cost_center: Option<String>,
    pub profit_center: Option<String>,
    pub segment: Option<String>,
    pub internal_order: Option<String>,
    pub wbs_element: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocation_sender_supports_cost_objects() {
        let sender = AllocationSender {
            sender_id: Uuid::new_v4(),
            run_id: Uuid::new_v4(),
            sender_object: "CCTR-1000".to_string(),
            sent_amount: Decimal::new(5000, 2),
            currency: "CNY".to_string(),
            cost_center: Some("CCTR-1000".to_string()),
            profit_center: Some("PCTR-01".to_string()),
            segment: None,
            internal_order: None,
            wbs_element: None,
        };

        assert_eq!(sender.cost_center.as_deref(), Some("CCTR-1000"));
        assert_eq!(sender.profit_center.as_deref(), Some("PCTR-01"));
    }

    #[test]
    fn allocation_receiver_supports_cost_objects() {
        let receiver = AllocationReceiver {
            receiver_id: Uuid::new_v4(),
            run_id: Uuid::new_v4(),
            receiver_object: "CCTR-2000".to_string(),
            received_amount: Decimal::new(5000, 2),
            currency: "CNY".to_string(),
            cost_center: Some("CCTR-2000".to_string()),
            profit_center: None,
            segment: None,
            internal_order: None,
            wbs_element: None,
        };

        assert_eq!(receiver.cost_center.as_deref(), Some("CCTR-2000"));
    }
}
