use sqlx::PgPool; use crate::domain::VendorEvaluation; use anyhow::Result;
pub struct VendorRepository { pool: PgPool }
impl VendorRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
    pub async fn save(&self, e: &VendorEvaluation) -> Result<()> {
        sqlx::query!("INSERT INTO vendor_evaluations (eval_id, vendor_id, evaluation_date, overall_score, quality_score, delivery_score, price_score, status) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
            e.eval_id, e.vendor_id, e.evaluation_date, e.overall_score, e.quality_score, e.delivery_score, e.price_score, e.status
        ).execute(&self.pool).await?; Ok(())
    }
    pub async fn find_by_vendor(&self, vendor_id: &str) -> Result<Vec<VendorEvaluation>> {
        let rows = sqlx::query!("SELECT * FROM vendor_evaluations WHERE vendor_id = $1 ORDER BY evaluation_date DESC", vendor_id).fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| VendorEvaluation { eval_id: r.eval_id, vendor_id: r.vendor_id, evaluation_date: r.evaluation_date, overall_score: r.overall_score, quality_score: r.quality_score, delivery_score: r.delivery_score, price_score: r.price_score, status: r.status.unwrap_or_default(), created_at: r.created_at }).collect())
    }
}
