use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProductionOrder {
    pub order_id: Uuid,
    pub order_number: String,
    pub order_type: String,
    pub material: String,
    pub plant: String,
    pub total_quantity: Decimal,
    pub delivered_quantity: Decimal,
    pub quantity_unit: String,
    pub basic_start_date: NaiveDate,
    pub basic_finish_date: NaiveDate,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Aggregates
    #[sqlx(skip)]
    pub operations: Vec<ProductionOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProductionOperation {
    pub operation_id: Uuid,
    pub order_id: Uuid,
    pub operation_number: String,
    pub work_center: String,
    pub description: Option<String>,
    pub confirmed_yield: Decimal,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProductionConfirmation {
    pub confirmation_id: Uuid,
    pub confirmation_number: String,
    pub order_id: Uuid,
    pub operation_number: String,
    pub yield_quantity: Decimal,
    pub scrap_quantity: Decimal,
    pub final_confirmation: bool,
    pub posting_date: NaiveDate,
    pub personnel_number: Option<String>,
    pub created_at: DateTime<Utc>,
}
