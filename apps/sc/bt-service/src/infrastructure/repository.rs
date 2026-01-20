use crate::domain::{Batch, BatchHistoryEvent};
use anyhow::Result;
use sqlx::PgPool;

pub struct BatchRepository {
    pool: PgPool,
}

impl BatchRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, b: &Batch) -> Result<()> {
        sqlx::query(
            "INSERT INTO batches (batch_id, batch_number, material, plant, production_date, expiration_date, supplier_batch, origin_batch, status) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
            .bind(b.batch_id)
            .bind(&b.batch_number)
            .bind(&b.material)
            .bind(&b.plant)
            .bind(b.production_date)
            .bind(b.expiration_date)
            .bind(&b.supplier_batch)
            .bind(&b.origin_batch)
            .bind(&b.status)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn find_by_material_batch(
        &self,
        mat: &str,
        batch: &str,
        plant: &str,
    ) -> Result<Option<Batch>> {
        let h = sqlx::query_as::<_, Batch>(
            "SELECT * FROM batches WHERE material = $1 AND batch_number = $2 AND plant = $3",
        )
        .bind(mat)
        .bind(batch)
        .bind(plant)
        .fetch_optional(&self.pool)
        .await?;
        Ok(h)
    }

    pub async fn add_history(&self, e: &BatchHistoryEvent) -> Result<()> {
        sqlx::query(
            "INSERT INTO batch_history (event_id, batch_id, event_type, user_id, details, document_number, document_type) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(e.event_id)
            .bind(e.batch_id)
            .bind(&e.event_type)
            .bind(&e.user_id)
            .bind(&e.details)
            .bind(&e.document_number)
            .bind(&e.document_type)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_history(&self, batch_id: uuid::Uuid) -> Result<Vec<BatchHistoryEvent>> {
        let rows = sqlx::query_as::<_, BatchHistoryEvent>(
            "SELECT * FROM batch_history WHERE batch_id = $1 ORDER BY event_time",
        )
        .bind(batch_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
}
