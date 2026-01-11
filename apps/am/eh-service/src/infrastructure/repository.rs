use sqlx::PgPool; use crate::domain::Incident; use anyhow::Result;
pub struct IncidentRepository { pool: PgPool }
impl IncidentRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
    pub async fn save(&self, i: &Incident) -> Result<()> {
        sqlx::query!("INSERT INTO incidents (incident_id, incident_code, category, title, description, location, incident_datetime, reported_by, status) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
            i.incident_id, i.incident_code, i.category, i.title, i.description, i.location, i.incident_datetime, i.reported_by, i.status
        ).execute(&self.pool).await?; Ok(())
    }
    pub async fn find_by_code(&self, code: &str) -> Result<Option<Incident>> {
        let r = sqlx::query!("SELECT * FROM incidents WHERE incident_code = $1", code).fetch_optional(&self.pool).await?;
        Ok(r.map(|r| Incident { incident_id: r.incident_id, incident_code: r.incident_code, category: r.category, title: r.title, description: r.description, location: r.location, incident_datetime: r.incident_datetime, reported_by: r.reported_by, status: r.status.unwrap_or_default(), created_at: r.created_at }))
    }
}
