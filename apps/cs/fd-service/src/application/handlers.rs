use crate::application::commands::{
    AssignTechnicianCommand, CreateOrderCommand, UpdateStatusCommand,
};
use crate::domain::ServiceOrder;
use crate::infrastructure::repository::ServiceOrderRepository;
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct ServiceHandler {
    repo: Arc<ServiceOrderRepository>,
}

impl ServiceHandler {
    pub fn new(repo: Arc<ServiceOrderRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_order(&self, cmd: CreateOrderCommand) -> Result<String> {
        let order_num = format!("SO{}", Utc::now().timestamp_subsec_micros());
        let o = ServiceOrder {
            order_id: Uuid::new_v4(),
            order_number: order_num.clone(),
            order_type: "REPAIR".to_string(),
            customer_id: cmd.customer_id,
            description: Some(cmd.description),
            planned_start: None,
            assigned_technician_id: None,
            status: "OPEN".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.repo.create(&o).await?;
        Ok(order_num)
    }

    pub async fn assign_technician(&self, cmd: AssignTechnicianCommand) -> Result<()> {
        self.repo
            .assign_technician(&cmd.order_number, &cmd.technician_id, cmd.scheduled_time)
            .await
    }

    pub async fn update_status(&self, cmd: UpdateStatusCommand) -> Result<()> {
        self.repo
            .update_status(&cmd.order_number, &cmd.new_status)
            .await
    }
}
