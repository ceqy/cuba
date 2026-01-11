use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastPlan {
    pub plan_id: Uuid,
    pub plan_code: String,
    pub material: String,
    pub plant: String,
    pub forecast_version: Option<String>,
    pub model_used: Option<String>,
    pub created_at: DateTime<Utc>,
    pub periods: Vec<ForecastPeriod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastPeriod {
    pub period_id: Uuid,
    pub plan_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub forecasted_quantity: Option<Decimal>,
    pub unit: String,
    pub confidence_lower: Option<Decimal>,
    pub confidence_upper: Option<Decimal>,
}
