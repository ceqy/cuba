use serde::Deserialize;
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
pub struct CreateJobApplicationCommand {
    pub requisition_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub resume_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusCommand {
    pub application_id: String,
    pub new_status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ScheduleInterviewCommand {
    pub application_id: String,
    pub interview_type: String,
    pub scheduled_time: DateTime<Utc>,
    pub interviewer_id: Option<String>,
    pub location: Option<String>,
}
