use crate::domain::{Recognition, Survey, SurveyResponse};
use anyhow::Result;
use sqlx::PgPool;

pub struct ExperienceRepository {
    pool: PgPool,
}

impl ExperienceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_survey(&self, s: &Survey) -> Result<()> {
        sqlx::query(
            "INSERT INTO surveys (survey_id, title, target_audience, status, created_at) VALUES ($1, $2, $3, $4, $5)")
            .bind(s.survey_id)
            .bind(&s.title)
            .bind(&s.target_audience)
            .bind(&s.status)
            .bind(s.created_at)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn submit_response(&self, r: &SurveyResponse) -> Result<()> {
        sqlx::query(
            "INSERT INTO survey_responses (response_id, survey_id, employee_id, answers, submitted_at) VALUES ($1, $2, $3, $4, $5)")
            .bind(r.response_id)
            .bind(r.survey_id)
            .bind(&r.employee_id)
            .bind(&r.answers)
            .bind(r.submitted_at)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn create_recognition(&self, r: &Recognition) -> Result<()> {
        sqlx::query(
            "INSERT INTO recognitions (recognition_id, giver_employee_id, receiver_employee_id, message, company_value, status, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(r.recognition_id)
            .bind(&r.giver_employee_id)
            .bind(&r.receiver_employee_id)
            .bind(&r.message)
            .bind(&r.company_value)
            .bind(&r.status)
            .bind(r.created_at)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn list_recognitions(&self, employee_id: &str) -> Result<Vec<Recognition>> {
        let rows = sqlx::query_as::<_, Recognition>("SELECT recognition_id, giver_employee_id, receiver_employee_id, message, company_value, status, created_at FROM recognitions WHERE receiver_employee_id = $1 ORDER BY created_at DESC")
            .bind(employee_id)
            .fetch_all(&self.pool).await?;
        Ok(rows)
    }
}
