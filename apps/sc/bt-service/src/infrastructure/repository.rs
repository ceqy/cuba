use sqlx::PgPool;
use crate::domain::{Batch, BatchHistoryEvent};
use anyhow::Result;

pub struct BatchRepository {
    pool: PgPool,
}

impl BatchRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, b: &Batch) -> Result<()> {
        sqlx::query!(
            "INSERT INTO batches (batch_id, batch_number, material, plant, production_date, expiration_date, supplier_batch, origin_batch, status) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            b.batch_id, b.batch_number, b.material, b.plant, b.production_date, b.expiration_date, b.supplier_batch, b.origin_batch, b.status
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn find_by_material_batch(&self, mat: &str, batch: &str, plant: &str) -> Result<Option<Batch>> {
        let h = sqlx::query!("SELECT * FROM batches WHERE material = $1 AND batch_number = $2 AND plant = $3", mat, batch, plant)
            .fetch_optional(&self.pool).await?;
        Ok(h.map(|h| Batch {
            batch_id: h.batch_id,
            batch_number: h.batch_number,
            material: h.material,
            plant: h.plant,
            production_date: h.production_date,
            expiration_date: h.expiration_date,
            supplier_batch: h.supplier_batch,
            origin_batch: h.origin_batch,
            status: h.status.unwrap_or_default(),
            created_at: h.created_at,
        }))
    }

    pub async fn add_history(&self, e: &BatchHistoryEvent) -> Result<()> {
        sqlx::query!(
            "INSERT INTO batch_history (event_id, batch_id, event_type, user_id, details, document_number, document_type) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            e.event_id, e.batch_id, e.event_type, e.user_id, e.details, e.document_number, e.document_type
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_history(&self, batch_id: uuid::Uuid) -> Result<Vec<BatchHistoryEvent>> {
        let rows = sqlx::query!("SELECT * FROM batch_history WHERE batch_id = $1 ORDER BY event_time", batch_id)
            .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| BatchHistoryEvent {
            event_id: r.event_id,
            batch_id: r.batch_id,
            event_time: r.event_time,
            event_type: r.event_type,
            user_id: r.user_id,
            details: r.details,
            document_number: r.document_number,
            document_type: r.document_type,
        }).collect())
    }
}
