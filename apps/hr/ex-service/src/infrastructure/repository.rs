use sqlx::PgPool;
use crate::domain::{Survey, SurveyResponse, Recognition};
use anyhow::Result;

pub struct ExperienceRepository {
    pool: PgPool,
}

impl ExperienceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_survey(&self, s: &Survey) -> Result<()> {
        sqlx::query!(
            "INSERT INTO surveys (survey_id, title, target_audience, status, created_at) VALUES ($1, $2, $3, $4, $5)",
            s.survey_id, s.title, s.target_audience, s.status, s.created_at
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn submit_response(&self, r: &SurveyResponse) -> Result<()> {
        sqlx::query!(
            "INSERT INTO survey_responses (response_id, survey_id, employee_id, answers, submitted_at) VALUES ($1, $2, $3, $4, $5)",
            r.response_id, r.survey_id, r.employee_id, r.answers, r.submitted_at
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn create_recognition(&self, r: &Recognition) -> Result<()> {
        sqlx::query!(
            "INSERT INTO recognitions (recognition_id, giver_employee_id, receiver_employee_id, message, company_value, status, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            r.recognition_id, r.giver_employee_id, r.receiver_employee_id, r.message, r.company_value, r.status, r.created_at
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn list_recognitions(&self, employee_id: &str) -> Result<Vec<Recognition>> {
        let rows = sqlx::query!("SELECT * FROM recognitions WHERE receiver_employee_id = $1 ORDER BY created_at DESC", employee_id)
            .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| Recognition {
            recognition_id: r.recognition_id,
            giver_employee_id: r.giver_employee_id,
            receiver_employee_id: r.receiver_employee_id,
            message: r.message,
            company_value: r.company_value,
            status: r.status.unwrap_or_else(|| "ACTIVE".to_string()),
            created_at: r.created_at,
        }).collect())
    }
}
