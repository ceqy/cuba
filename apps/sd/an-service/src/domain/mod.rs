use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SalesDimension {
    pub id: String,
    pub name: String,
    pub revenue: Decimal,
    pub currency: String,
    pub quantity_sold: Decimal,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TimeSeriesDataPoint {
    pub period: String,
    pub revenue: Decimal,
    pub currency: String,
}
