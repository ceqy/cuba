use sqlx::PgPool; use crate::domain::VendorEvaluation; use anyhow::Result;
pub struct VendorRepository { pool: PgPool }
impl VendorRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
    pub async fn save(&self, e: &VendorEvaluation) -> Result<()> {
        sqlx::query("INSERT INTO vendor_evaluations (eval_id, vendor_id, evaluation_date, overall_score, quality_score, delivery_score, price_score, status) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)")
            .bind(e.eval_id)
            .bind(&e.vendor_id)
            .bind(e.evaluation_date)
            .bind(e.overall_score)
            .bind(e.quality_score)
            .bind(e.delivery_score)
            .bind(e.price_score)
            .bind(&e.status)
        .execute(&self.pool).await?; Ok(())
    }
    pub async fn find_by_vendor(&self, vendor_id: &str) -> Result<Vec<VendorEvaluation>> {
        let rows = sqlx::query_as::<_, VendorEvaluation>("SELECT eval_id, vendor_id, evaluation_date, overall_score, quality_score, delivery_score, price_score, status, created_at FROM vendor_evaluations WHERE vendor_id = $1 ORDER BY evaluation_date DESC")
            .bind(vendor_id)
            .fetch_all(&self.pool).await?;
        Ok(rows)
    }
}
