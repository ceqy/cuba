use std::sync::Arc;
use crate::domain::{JobApplication, InterviewSchedule};
use crate::infrastructure::repository::TalentRepository;
use crate::application::commands::{CreateJobApplicationCommand, UpdateStatusCommand, ScheduleInterviewCommand};
use anyhow::{Result, anyhow};
use uuid::Uuid;
use chrono::Utc;

pub struct TalentHandler {
    repo: Arc<TalentRepository>,
}

impl TalentHandler {
    pub fn new(repo: Arc<TalentRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_application(&self, cmd: CreateJobApplicationCommand) -> Result<String> {
        let candidate_id = self.repo.find_or_create_candidate(
            &cmd.email, &cmd.first_name, &cmd.last_name, cmd.phone.as_deref(), cmd.resume_url.as_deref()
        ).await?;

        let app_id = Uuid::new_v4();
        let app = JobApplication {
            application_id: app_id,
            requisition_id: cmd.requisition_id,
            requisition_title: None,
            candidate_id,
            status: "SUBMITTED".to_string(),
            application_date: Utc::now(),
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            candidate: None,
            interviews: vec![],
        };
        self.repo.create_application(&app).await?;
        Ok(app_id.to_string())
    }

    pub async fn update_status(&self, cmd: UpdateStatusCommand) -> Result<()> {
        let app_id = Uuid::parse_str(&cmd.application_id)?;
        self.repo.update_status(app_id, &cmd.new_status, cmd.notes.as_deref()).await
    }

    pub async fn schedule_interview(&self, cmd: ScheduleInterviewCommand) -> Result<String> {
        let app_id = Uuid::parse_str(&cmd.application_id)?;
        let _ = self.repo.find_application_by_id(app_id).await?
            .ok_or_else(|| anyhow!("Application not found"))?;

        let interview_id = Uuid::new_v4();
        let interview = InterviewSchedule {
            interview_id,
            application_id: app_id,
            interview_type: cmd.interview_type,
            scheduled_time: cmd.scheduled_time,
            interviewer_id: cmd.interviewer_id,
            location: cmd.location,
            notes: None,
        };
        self.repo.schedule_interview(&interview).await?;
        Ok(interview_id.to_string())
    }
}
