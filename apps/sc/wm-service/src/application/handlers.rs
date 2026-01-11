use std::sync::Arc;
use crate::domain::{TransferOrder, TransferOrderItem};
use crate::infrastructure::repository::TransferOrderRepository;
use crate::application::commands::{CreateTOCommand, ConfirmTOCommand};
use anyhow::{Result, anyhow};
use uuid::Uuid;
use chrono::Utc;
use rust_decimal::Decimal;

pub struct WarehouseHandler {
    repo: Arc<TransferOrderRepository>,
}

impl WarehouseHandler {
    pub fn new(repo: Arc<TransferOrderRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_transfer_order(&self, cmd: CreateTOCommand) -> Result<String> {
        let to_id = Uuid::new_v4();
        let to_number = format!("TO{}", Utc::now().timestamp_subsec_micros());
        let to = TransferOrder {
            to_id,
            to_number: to_number.clone(),
            warehouse_number: cmd.warehouse_number,
            movement_type: cmd.movement_type,
            reference_doc_type: Some("TR".to_string()),
            reference_doc_number: cmd.reference_doc_number,
            status: "CREATED".to_string(),
            created_by: Some("SYSTEM".to_string()),
            created_at: Utc::now(),
            items: vec![TransferOrderItem {
                item_id: Uuid::new_v4(),
                to_id,
                item_number: 10,
                material: "MAT-001".to_string(),
                target_quantity: Decimal::new(100, 0),
                actual_quantity: Decimal::ZERO,
                unit: "EA".to_string(),
                src_storage_type: Some("01".to_string()),
                src_storage_bin: Some("A-01-01".to_string()),
                dst_storage_type: Some("02".to_string()),
                dst_storage_bin: Some("B-01-01".to_string()),
                batch: None,
                confirmed: false,
            }],
        };
        self.repo.save(&to).await?;
        Ok(to_number)
    }

    pub async fn confirm_transfer_order(&self, cmd: ConfirmTOCommand) -> Result<()> {
        let to = self.repo.find_by_number(&cmd.warehouse_number, &cmd.to_number).await?
            .ok_or_else(|| anyhow!("TO not found"))?;
        self.repo.confirm_order(to.to_id).await
    }
}
