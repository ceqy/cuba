use std::sync::Arc;
use crate::domain::PlannedOrder;
use crate::infrastructure::repository::PlannedOrderRepository;
use crate::application::commands::RunMrpCommand;
use anyhow::Result;
use uuid::Uuid;
use chrono::{Utc, Duration};
use rust_decimal::Decimal;

pub struct RunMrpHandler {
    repo: Arc<PlannedOrderRepository>,
}

impl RunMrpHandler {
    pub fn new(repo: Arc<PlannedOrderRepository>) -> Self {
        Self { repo }
    }

    pub async fn handle(&self, cmd: RunMrpCommand) -> Result<String> {
        let job_id = Uuid::new_v4().to_string();
        
        // MVP: Simulate creating 1 planned order for each material
        for mat in &cmd.materials {
            let plaf = PlannedOrder {
                planned_order_id: Uuid::new_v4(),
                planned_order_number: format!("001{}", Utc::now().timestamp_subsec_micros()),
                material: mat.clone(),
                plant: cmd.plant.clone(),
                planning_plant: cmd.plant.clone(),
                order_quantity: Decimal::from(100),
                quantity_unit: "PC".to_string(),
                order_start_date: Utc::now().date_naive(),
                order_finish_date: (Utc::now() + Duration::days(5)).date_naive(),
                mrp_controller: Some("001".to_string()),
                conversion_indicator: "".to_string(),
                status: "CREATED".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            self.repo.save(&plaf).await?;
        }

        Ok(job_id)
    }
}
