use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ServiceContract {
    pub contract_id: Uuid,
    pub contract_number: String,
    pub customer_id: String,
    pub validity_start: NaiveDate,
    pub validity_end: NaiveDate,
    pub created_at: DateTime<Utc>,
    #[sqlx(skip)]
    pub billing_plan: Vec<BillingPlanItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BillingPlanItem {
    pub item_id: Uuid,
    pub contract_id: Uuid,
    pub planned_date: NaiveDate,
    pub amount: Decimal,
    pub currency: String,
    pub status: String,
    pub invoice_number: Option<String>,
}
