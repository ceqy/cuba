use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VendorEvaluation {
    pub eval_id: Uuid,
    pub vendor_id: String,
    pub evaluation_date: Option<NaiveDate>,
    pub overall_score: Option<Decimal>,
    pub quality_score: Option<Decimal>,
    pub delivery_score: Option<Decimal>,
    pub price_score: Option<Decimal>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
