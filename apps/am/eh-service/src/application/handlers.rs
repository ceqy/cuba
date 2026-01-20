use crate::domain::Incident;
use crate::infrastructure::repository::IncidentRepository;
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;
pub struct IncidentHandler {
    repo: Arc<IncidentRepository>,
}
impl IncidentHandler {
    pub fn new(repo: Arc<IncidentRepository>) -> Self {
        Self { repo }
    }
    pub async fn report(&self, title: String, desc: String) -> Result<String> {
        let id = Uuid::new_v4();
        let code = format!("INC{}", Utc::now().timestamp_subsec_micros());
        let i = Incident {
            incident_id: id,
            incident_code: code.clone(),
            category: Some("SAFETY".to_string()),
            title: Some(title),
            description: Some(desc),
            location: None,
            incident_datetime: Some(Utc::now()),
            reported_by: Some("SYSTEM".to_string()),
            status: "REPORTED".to_string(),
            created_at: Utc::now(),
        };
        self.repo.save(&i).await?;
        Ok(code)
    }
}
