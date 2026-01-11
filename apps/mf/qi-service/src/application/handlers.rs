use std::sync::Arc;
use crate::domain::{InspectionLot, InspectionCharacteristic};
use crate::infrastructure::repository::InspectionLotRepository;
use crate::application::commands::{CreateInspectionLotCommand, RecordResultCommand, MakeUsageDecisionCommand};
use anyhow::{Result, anyhow};
use uuid::Uuid;
use chrono::Utc;

pub struct InspectionHandler {
    repo: Arc<InspectionLotRepository>,
}

impl InspectionHandler {
    pub fn new(repo: Arc<InspectionLotRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_lot(&self, cmd: CreateInspectionLotCommand) -> Result<String> {
        let lot_id = Uuid::new_v4();
        let lot_number = format!("80{}", Utc::now().timestamp_subsec_micros()); 

        let origin_str = cmd.origin.to_string(); // Simplified mapping

        let lot = InspectionLot {
            lot_id,
            inspection_lot_number: lot_number.clone(),
            material: cmd.material,
            plant: cmd.plant,
            lot_quantity: cmd.quantity,
            quantity_unit: "PC".to_string(), // Simplified
            origin: origin_str,
            creation_date: Utc::now(),
            ud_code: None,
            ud_date: None,
            ud_note: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            characteristics: vec![
                InspectionCharacteristic {
                    char_id: Uuid::new_v4(),
                    lot_id,
                    characteristic_number: "0010".to_string(),
                    description: Some("Visual Inspection".to_string()),
                    inspection_method: Some("Visual".to_string()),
                    result_value: None,
                    result_status: "0".to_string(),
                },
                InspectionCharacteristic {
                     char_id: Uuid::new_v4(),
                    lot_id,
                    characteristic_number: "0020".to_string(),
                    description: Some("Dimensions Check".to_string()),
                    inspection_method: Some("Caliper".to_string()),
                    result_value: None,
                    result_status: "0".to_string(),
                }
            ],
        };

        self.repo.create_lot(&lot).await?;
        Ok(lot_number)
    }

    pub async fn record_result(&self, cmd: RecordResultCommand) -> Result<()> {
        let lot = self.repo.find_by_number(&cmd.lot_number).await?
            .ok_or_else(|| anyhow!("Lot not found"))?;

        self.repo.update_result(lot.lot_id, &cmd.characteristic_number, &cmd.value).await?;
        Ok(())
    }

    pub async fn make_usage_decision(&self, cmd: MakeUsageDecisionCommand) -> Result<()> {
         let lot = self.repo.find_by_number(&cmd.lot_number).await?
            .ok_or_else(|| anyhow!("Lot not found"))?;
            
         self.repo.make_usage_decision(lot.lot_id, &cmd.ud_code, &cmd.note).await?;
         Ok(())
    }
}
