use crate::application::commands::IngestDataCommand;
use crate::domain::SensorDataPoint;
use crate::infrastructure::repository::HealthRepository;
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct HealthHandler {
    repo: Arc<HealthRepository>,
}

impl HealthHandler {
    pub fn new(repo: Arc<HealthRepository>) -> Self {
        Self { repo }
    }

    pub async fn ingest_data(&self, cmd: IngestDataCommand) -> Result<String> {
        let data = SensorDataPoint {
            data_id: Uuid::new_v4(),
            equipment_number: cmd.equipment_number,
            sensor_id: cmd.sensor_id,
            value: Some(cmd.value),
            unit: Some("UNIT".to_string()),
            recorded_at: Utc::now(),
        };
        self.repo.save_sensor_data(&data).await?;
        Ok(format!("JOB{}", Utc::now().timestamp_subsec_micros()))
    }
}
