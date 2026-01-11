use std::sync::Arc;
use crate::domain::{MaintenanceNotification, MaintenanceOrder, MaintenanceOperation};
use crate::infrastructure::repository::MaintenanceRepository;
use crate::application::commands::{CreateNotificationCommand, CreateOrderCommand, ConfirmOperationCommand};
use anyhow::{Result, anyhow};
use uuid::Uuid;
use chrono::Utc;
use rust_decimal::Decimal;

pub struct MaintenanceHandler {
    repo: Arc<MaintenanceRepository>,
}

impl MaintenanceHandler {
    pub fn new(repo: Arc<MaintenanceRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_notification(&self, cmd: CreateNotificationCommand) -> Result<String> {
        let notif_num = format!("1000{}", Utc::now().timestamp_subsec_micros());
        let n = MaintenanceNotification {
            notification_id: Uuid::new_v4(),
            notification_number: notif_num.clone(),
            notification_type: cmd.notification_type,
            description: Some(cmd.description),
            equipment_number: Some(cmd.equipment_number),
            functional_location: None,
            reported_by: Some("USER".to_string()),
            reported_date: Some(Utc::now()),
            priority: "3".to_string(),
            status: "OSNO".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.repo.create_notification(&n).await?;
        Ok(notif_num)
    }

    pub async fn create_order(&self, cmd: CreateOrderCommand) -> Result<String> {
        let order_id = Uuid::new_v4();
        let order_num = format!("4000{}", Utc::now().timestamp_subsec_micros());
        let o = MaintenanceOrder {
            order_id,
            order_number: order_num.clone(),
            order_type: cmd.order_type,
            description: Some(cmd.description),
            notification_number: None,
            equipment_number: Some(cmd.equipment_number),
            functional_location: None,
            maintenance_plant: cmd.maintenance_plant,
            planning_plant: None,
            main_work_center: Some("WC01".to_string()),
            system_status: "CRTD".to_string(),
            priority: "3".to_string(),
            basic_start_date: Some(Utc::now().date_naive()),
            basic_finish_date: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            operations: vec![
                MaintenanceOperation {
                    operation_id: Uuid::new_v4(),
                    order_id,
                    operation_number: "0010".to_string(),
                    description: Some("Repair Task".to_string()),
                    work_center: Some("WC01".to_string()),
                    planned_work_duration: Decimal::from(4),
                    actual_work_duration: Decimal::ZERO,
                    work_unit: "H".to_string(),
                    status: "CRTD".to_string(),
                }
            ],
        };
        self.repo.create_order(&o).await?;
        Ok(order_num)
    }

    pub async fn confirm_operation(&self, cmd: ConfirmOperationCommand) -> Result<String> {
        let order = self.repo.find_order_by_number(&cmd.order_number).await?
            .ok_or_else(|| anyhow!("Order not found"))?;
        self.repo.confirm_operation(order.order_id, &cmd.operation_number, cmd.actual_duration).await?;
        Ok(format!("CNF{}", Utc::now().timestamp_subsec_micros()))
    }
}
