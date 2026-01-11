use std::sync::Arc;
use crate::domain::{Shipment, ShipmentItem};
use crate::infrastructure::repository::ShipmentRepository;
use crate::application::commands::{CreateShipmentCommand, UpdateStatusCommand};
use anyhow::Result;
use uuid::Uuid;
use chrono::Utc;

pub struct ShipmentHandler {
    repo: Arc<ShipmentRepository>,
}

impl ShipmentHandler {
    pub fn new(repo: Arc<ShipmentRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_shipment(&self, cmd: CreateShipmentCommand) -> Result<String> {
        let shipment_id = Uuid::new_v4();
        let shipment_number = format!("SHP{}", Utc::now().timestamp_subsec_micros());
        let items = cmd.delivery_numbers.iter().enumerate().map(|(idx, dn)| ShipmentItem {
            item_id: Uuid::new_v4(),
            shipment_id,
            item_number: (idx as i32 + 1) * 10,
            delivery_number: dn.clone(),
            total_weight: Some(rust_decimal::Decimal::new(100, 0)),
            weight_unit: "KG".to_string(),
            volume: Some(rust_decimal::Decimal::new(1, 0)),
            volume_unit: "M3".to_string(),
        }).collect();
        let s = Shipment {
            shipment_id, shipment_number: shipment_number.clone(),
            shipment_type: Some("OUTBOUND".to_string()),
            transportation_planning_point: Some(cmd.planning_point),
            carrier: None,
            overall_status: "PLANNED".to_string(),
            planned_departure: Some(Utc::now() + chrono::Duration::days(1)),
            planned_arrival: Some(Utc::now() + chrono::Duration::days(3)),
            created_at: Utc::now(),
            items,
        };
        self.repo.save(&s).await?;
        Ok(shipment_number)
    }

    pub async fn update_status(&self, cmd: UpdateStatusCommand) -> Result<bool> {
        self.repo.update_status(&cmd.shipment_number, &cmd.new_status).await?;
        Ok(true)
    }
}
