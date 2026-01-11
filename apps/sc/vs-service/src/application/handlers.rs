use std::sync::Arc; use crate::domain::VendorEvaluation; use crate::infrastructure::repository::VendorRepository; use anyhow::Result; use uuid::Uuid; use chrono::Utc;
pub struct VendorHandler { repo: Arc<VendorRepository> }
impl VendorHandler {
    pub fn new(repo: Arc<VendorRepository>) -> Self { Self { repo } }
    pub async fn evaluate(&self, vendor_id: String) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let e = VendorEvaluation { eval_id: id, vendor_id, evaluation_date: Some(Utc::now().date_naive()), overall_score: Some(rust_decimal::Decimal::new(85, 0)), quality_score: Some(rust_decimal::Decimal::new(90, 0)), delivery_score: Some(rust_decimal::Decimal::new(85, 0)), price_score: Some(rust_decimal::Decimal::new(80, 0)), status: "ACTIVE".to_string(), created_at: Utc::now() };
        self.repo.save(&e).await?; Ok(id)
    }
}
