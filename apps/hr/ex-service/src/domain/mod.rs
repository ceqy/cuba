use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Survey {
    pub survey_id: Uuid,
    pub title: String,
    pub target_audience: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurveyResponse {
    pub response_id: Uuid,
    pub survey_id: Uuid,
    pub employee_id: String,
    pub answers: Option<serde_json::Value>,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recognition {
    pub recognition_id: Uuid,
    pub giver_employee_id: String,
    pub receiver_employee_id: String,
    pub message: Option<String>,
    pub company_value: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
