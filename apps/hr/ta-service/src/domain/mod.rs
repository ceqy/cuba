use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub candidate_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub resume_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobApplication {
    pub application_id: Uuid,
    pub requisition_id: String,
    pub requisition_title: Option<String>,
    pub candidate_id: Uuid,
    pub status: String,
    pub application_date: DateTime<Utc>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub candidate: Option<Candidate>,
    pub interviews: Vec<InterviewSchedule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterviewSchedule {
    pub interview_id: Uuid,
    pub application_id: Uuid,
    pub interview_type: String,
    pub scheduled_time: DateTime<Utc>,
    pub interviewer_id: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
}
