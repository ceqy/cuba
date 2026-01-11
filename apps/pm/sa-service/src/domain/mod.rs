use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendDimension {
    pub id: String,
    pub name: String,
    pub spend_amount: Decimal,
    pub currency: String,
    pub document_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesDataPoint {
    pub period: String,
    pub spend_amount: Decimal,
    pub currency: String,
}
