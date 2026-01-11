use sqlx::PgPool;
use crate::domain::{Candidate, JobApplication, InterviewSchedule};
use anyhow::Result;

pub struct TalentRepository {
    pool: PgPool,
}

impl TalentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_or_create_candidate(&self, email: &str, first_name: &str, last_name: &str, phone: Option<&str>, resume_url: Option<&str>) -> Result<uuid::Uuid> {
        let existing = sqlx::query!("SELECT candidate_id FROM candidates WHERE email = $1", email)
            .fetch_optional(&self.pool).await?;
        if let Some(c) = existing {
            return Ok(c.candidate_id);
        }
        let id = uuid::Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO candidates (candidate_id, first_name, last_name, email, phone, resume_url) VALUES ($1, $2, $3, $4, $5, $6)",
            id, first_name, last_name, email, phone, resume_url
        ).execute(&self.pool).await?;
        Ok(id)
    }

    pub async fn create_application(&self, app: &JobApplication) -> Result<()> {
        sqlx::query!(
            r#"INSERT INTO job_applications (application_id, requisition_id, requisition_title, candidate_id, status, application_date, notes, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
            app.application_id, app.requisition_id, app.requisition_title, app.candidate_id, app.status, app.application_date, app.notes, app.created_at, app.updated_at
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn find_application_by_id(&self, app_id: uuid::Uuid) -> Result<Option<JobApplication>> {
        let h = sqlx::query!("SELECT * FROM job_applications WHERE application_id = $1", app_id)
            .fetch_optional(&self.pool).await?;
        if let Some(h) = h {
            let interviews = sqlx::query!("SELECT * FROM interview_schedules WHERE application_id = $1", app_id)
                .fetch_all(&self.pool).await?
                .into_iter().map(|i| InterviewSchedule {
                    interview_id: i.interview_id,
                    application_id: i.application_id,
                    interview_type: i.interview_type,
                    scheduled_time: i.scheduled_time,
                    interviewer_id: i.interviewer_id,
                    location: i.location,
                    notes: i.notes,
                }).collect();
            Ok(Some(JobApplication {
                application_id: h.application_id,
                requisition_id: h.requisition_id,
                requisition_title: h.requisition_title,
                candidate_id: h.candidate_id,
                status: h.status.unwrap_or_else(|| "SUBMITTED".to_string()),
                application_date: h.application_date,
                notes: h.notes,
                created_at: h.created_at,
                updated_at: h.updated_at,
                candidate: None,
                interviews,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_status(&self, app_id: uuid::Uuid, status: &str, notes: Option<&str>) -> Result<()> {
        sqlx::query!("UPDATE job_applications SET status = $1, notes = COALESCE($2, notes), updated_at = NOW() WHERE application_id = $3",
            status, notes, app_id
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn schedule_interview(&self, interview: &InterviewSchedule) -> Result<()> {
        sqlx::query!(
            "INSERT INTO interview_schedules (interview_id, application_id, interview_type, scheduled_time, interviewer_id, location, notes) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            interview.interview_id, interview.application_id, interview.interview_type, interview.scheduled_time, interview.interviewer_id, interview.location, interview.notes
        ).execute(&self.pool).await?;
        Ok(())
    }
}
