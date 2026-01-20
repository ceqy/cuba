use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Claim {
    pub claim_id: Uuid,
    pub customer_id: String,
    pub product_id: String,
    pub failure_date: NaiveDate,
    pub failure_description: Option<String>,
    pub claimed_amount: Decimal,
    pub currency: String,
    pub status: String,
    pub attachment_urls: Vec<String>,
    pub created_at: DateTime<Utc>,
    #[sqlx(skip)]
    pub adjudication: Option<AdjudicationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AdjudicationResult {
    pub adjudication_id: Uuid,
    pub claim_id: Uuid,
    pub adjudicated_by: String,
    pub adjudication_date: DateTime<Utc>,
    pub approved_amount: Option<Decimal>,
    pub currency: String,
    pub notes: Option<String>,
}
