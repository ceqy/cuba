use crate::domain::{Candidate, InterviewSchedule, JobApplication};
use anyhow::Result;
use sqlx::PgPool;

pub struct TalentRepository {
    pool: PgPool,
}

impl TalentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_or_create_candidate(
        &self,
        email: &str,
        first_name: &str,
        last_name: &str,
        phone: Option<&str>,
        resume_url: Option<&str>,
    ) -> Result<uuid::Uuid> {
        let existing = sqlx::query("SELECT candidate_id FROM candidates WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;
        if let Some(r) = existing {
            use sqlx::Row;
            return Ok(r.get("candidate_id"));
        }
        let id = uuid::Uuid::new_v4();
        sqlx::query(
            "INSERT INTO candidates (candidate_id, first_name, last_name, email, phone, resume_url) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(id)
            .bind(first_name)
            .bind(last_name)
            .bind(email)
            .bind(phone)
            .bind(resume_url)
        .execute(&self.pool).await?;
        Ok(id)
    }

    pub async fn create_application(&self, app: &JobApplication) -> Result<()> {
        sqlx::query(
            "INSERT INTO job_applications (application_id, requisition_id, requisition_title, candidate_id, status, application_date, notes, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
            .bind(app.application_id)
            .bind(&app.requisition_id)
            .bind(&app.requisition_title)
            .bind(app.candidate_id)
            .bind(&app.status)
            .bind(app.application_date)
            .bind(&app.notes)
            .bind(app.created_at)
            .bind(app.updated_at)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn find_application_by_id(
        &self,
        app_id: uuid::Uuid,
    ) -> Result<Option<JobApplication>> {
        let h = sqlx::query_as::<_, JobApplication>("SELECT application_id, requisition_id, requisition_title, candidate_id, status, application_date, notes, created_at, updated_at FROM job_applications WHERE application_id = $1")
            .bind(app_id)
            .fetch_optional(&self.pool).await?;
        if let Some(mut h) = h {
            let interviews = sqlx::query_as::<_, InterviewSchedule>(
                "SELECT * FROM interview_schedules WHERE application_id = $1",
            )
            .bind(app_id)
            .fetch_all(&self.pool)
            .await?;
            h.interviews = interviews;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }

    pub async fn update_status(
        &self,
        app_id: uuid::Uuid,
        status: &str,
        notes: Option<&str>,
    ) -> Result<()> {
        sqlx::query("UPDATE job_applications SET status = $1, notes = COALESCE($2, notes), updated_at = NOW() WHERE application_id = $3")
            .bind(status)
            .bind(notes)
            .bind(app_id)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn schedule_interview(&self, interview: &InterviewSchedule) -> Result<()> {
        sqlx::query(
            "INSERT INTO interview_schedules (interview_id, application_id, interview_type, scheduled_time, interviewer_id, location, notes) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(interview.interview_id)
            .bind(interview.application_id)
            .bind(&interview.interview_type)
            .bind(interview.scheduled_time)
            .bind(&interview.interviewer_id)
            .bind(&interview.location)
            .bind(&interview.notes)
        .execute(&self.pool).await?;
        Ok(())
    }
}
