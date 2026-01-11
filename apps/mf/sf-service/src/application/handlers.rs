use std::sync::Arc;
use crate::domain::{ProductionOrder, ProductionOperation, ProductionConfirmation};
use crate::infrastructure::repository::ProductionOrderRepository;
use crate::application::commands::{CreateProductionOrderCommand, ConfirmOperationCommand};
use anyhow::{Result, anyhow};
use uuid::Uuid;
use chrono::{Utc, Duration};
use rust_decimal::Decimal;

pub struct ProductionHandler {
    repo: Arc<ProductionOrderRepository>,
}

impl ProductionHandler {
    pub fn new(repo: Arc<ProductionOrderRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_seed_order(&self, cmd: CreateProductionOrderCommand) -> Result<String> {
        let order_id = Uuid::new_v4();
        let order_number = format!("10{}", Utc::now().timestamp_subsec_micros()); // Mock number

        let order = ProductionOrder {
            order_id,
            order_number: order_number.clone(),
            order_type: "PP01".to_string(),
            material: cmd.material,
            plant: cmd.plant,
            total_quantity: cmd.quantity,
            delivered_quantity: Decimal::ZERO,
            quantity_unit: cmd.unit,
            basic_start_date: Utc::now().date_naive(),
            basic_finish_date: (Utc::now() + Duration::days(2)).date_naive(),
            status: "REL".to_string(), // Released immediately for MVP
            created_at: Utc::now(),
            updated_at: Utc::now(),
            operations: vec![
                ProductionOperation {
                    operation_id: Uuid::new_v4(),
                    order_id,
                    operation_number: "0010".to_string(),
                    work_center: "WC01".to_string(),
                    description: Some("Assembly".to_string()),
                    confirmed_yield: Decimal::ZERO,
                    status: "REL".to_string(),
                },
                ProductionOperation {
                    operation_id: Uuid::new_v4(),
                    order_id,
                    operation_number: "0020".to_string(),
                    work_center: "WC02".to_string(),
                    description: Some("Testing".to_string()),
                    confirmed_yield: Decimal::ZERO,
                    status: "REL".to_string(),
                }
            ],
        };

        self.repo.create_order(&order).await?;
        Ok(order_number)
    }

    pub async fn confirm_operation(&self, cmd: ConfirmOperationCommand) -> Result<String> {
        let order = self.repo.find_by_number(&cmd.order_number).await?
            .ok_or_else(|| anyhow!("Order not found"))?;

        let conf_number = format!("C{}", Utc::now().timestamp_subsec_micros());
        
        let conf = ProductionConfirmation {
            confirmation_id: Uuid::new_v4(),
            confirmation_number: conf_number.clone(),
            order_id: order.order_id,
            operation_number: cmd.operation_number,
            yield_quantity: cmd.yield_quantity,
            scrap_quantity: cmd.scrap_quantity,
            final_confirmation: cmd.final_confirmation,
            posting_date: cmd.posting_date,
            personnel_number: None,
            created_at: Utc::now(),
        };

        self.repo.save_confirmation(&conf).await?;
        Ok(conf_number)
    }
}
