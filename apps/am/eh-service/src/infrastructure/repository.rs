use crate::domain::Incident;
use anyhow::Result;
use sqlx::PgPool;
pub struct IncidentRepository {
    pool: PgPool,
}
impl IncidentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn save(&self, i: &Incident) -> Result<()> {
        sqlx::query(
            "INSERT INTO incidents (incident_id, incident_code, category, title, description, location, incident_datetime, reported_by, status) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)")
            .bind(i.incident_id)
            .bind(&i.incident_code)
            .bind(&i.category)
            .bind(&i.title)
            .bind(&i.description)
            .bind(&i.location)
            .bind(i.incident_datetime)
            .bind(&i.reported_by)
            .bind(&i.status)
        .execute(&self.pool).await?;
        Ok(())
    }
    pub async fn find_by_code(&self, code: &str) -> Result<Option<Incident>> {
        let r = sqlx::query_as::<_, Incident>("SELECT incident_id, incident_code, category, title, description, location, incident_datetime, reported_by, status, created_at FROM incidents WHERE incident_code = $1")
            .bind(code)
            .fetch_optional(&self.pool).await?;
        Ok(r)
    }
}
