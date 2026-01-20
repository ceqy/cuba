use crate::application::commands::{CreateBatchCommand, TraceCommand};
use crate::domain::{Batch, BatchHistoryEvent};
use crate::infrastructure::repository::BatchRepository;
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct BatchHandler {
    repo: Arc<BatchRepository>,
}

impl BatchHandler {
    pub fn new(repo: Arc<BatchRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_batch(&self, cmd: CreateBatchCommand) -> Result<String> {
        let batch_id = Uuid::new_v4();
        let batch_number = format!("B{}", Utc::now().timestamp_subsec_micros());
        let b = Batch {
            batch_id,
            batch_number: batch_number.clone(),
            material: cmd.material,
            plant: cmd.plant,
            production_date: cmd.production_date,
            expiration_date: cmd.expiration_date,
            supplier_batch: cmd.supplier_batch,
            origin_batch: None,
            status: "ACTIVE".to_string(),
            created_at: Utc::now(),
        };
        self.repo.save(&b).await?;

        // Log creation event
        let event = BatchHistoryEvent {
            event_id: Uuid::new_v4(),
            batch_id,
            event_time: Utc::now(),
            event_type: "CREATED".to_string(),
            user_id: Some("SYSTEM".to_string()),
            details: Some("Batch created".to_string()),
            document_number: None,
            document_type: None,
        };
        self.repo.add_history(&event).await?;

        Ok(batch_number)
    }

    pub async fn trace(&self, _cmd: TraceCommand, direction: &str) -> Result<String> {
        // Simplified: just return a job ID for now
        let job_id = format!("TRACE-{}-{}", direction, Uuid::new_v4());
        Ok(job_id)
    }
}
