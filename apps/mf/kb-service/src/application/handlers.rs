use std::sync::Arc;
use crate::domain::{ControlCycle, KanbanContainer};
use crate::infrastructure::repository::KanbanRepository;
use crate::application::commands::{CreateCycleCommand, ChangeStatusCommand};
use anyhow::{Result, anyhow};
use uuid::Uuid;
use chrono::Utc;

pub struct KanbanHandler {
    repo: Arc<KanbanRepository>,
}

impl KanbanHandler {
    pub fn new(repo: Arc<KanbanRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_cycle(&self, cmd: CreateCycleCommand) -> Result<String> {
        let cycle_id = Uuid::new_v4();
        let cycle_number = format!("KC{}", Utc::now().timestamp_subsec_micros());
        let c = ControlCycle {
            cycle_id,
            cycle_number: cycle_number.clone(),
            material: cmd.material,
            plant: cmd.plant,
            supply_area: cmd.supply_area,
            number_of_kanbans: cmd.number_of_kanbans,
            qty_per_kanban: cmd.qty_per_kanban,
            unit: "EA".to_string(),
            replenishment_strategy: "PRODUCTION".to_string(),
            created_at: Utc::now(),
        };
        self.repo.save_cycle(&c).await?;
        
        // Create kanban containers
        for i in 1..=cmd.number_of_kanbans {
            let k = KanbanContainer {
                container_id: Uuid::new_v4(),
                container_code: format!("{}-{:03}", cycle_number, i),
                cycle_id,
                status: "FULL".to_string(),
                last_status_change: Utc::now(),
            };
            self.repo.save_container(&k).await?;
        }
        
        Ok(cycle_number)
    }

    pub async fn change_status(&self, cmd: ChangeStatusCommand) -> Result<Option<String>> {
        let k = self.repo.find_container_by_code(&cmd.container_code).await?
            .ok_or_else(|| anyhow!("Container not found"))?;
        self.repo.update_container_status(k.container_id, &cmd.new_status).await?;
        
        // If status changed to EMPTY, trigger replenishment
        let replenishment_doc = if cmd.new_status == "EMPTY" {
            Some(format!("REP{}", Utc::now().timestamp_subsec_micros()))
        } else {
            None
        };
        Ok(replenishment_doc)
    }
}
