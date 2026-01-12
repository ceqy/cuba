use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MaintenanceNotification {
    pub notification_id: Uuid,
    pub notification_number: String,
    pub notification_type: String,
    pub description: Option<String>,
    pub equipment_number: Option<String>,
    pub functional_location: Option<String>,
    pub reported_by: Option<String>,
    pub reported_date: Option<DateTime<Utc>>,
    pub priority: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MaintenanceOrder {
    pub order_id: Uuid,
    pub order_number: String,
    pub order_type: String,
    pub description: Option<String>,
    pub notification_number: Option<String>,
    pub equipment_number: Option<String>,
    pub functional_location: Option<String>,
    pub maintenance_plant: String,
    pub planning_plant: Option<String>,
    pub main_work_center: Option<String>,
    pub system_status: String,
    pub priority: String,
    pub basic_start_date: Option<NaiveDate>,
    pub basic_finish_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[sqlx(skip)]
    pub operations: Vec<MaintenanceOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MaintenanceOperation {
    pub operation_id: Uuid,
    pub order_id: Uuid,
    pub operation_number: String,
    pub description: Option<String>,
    pub work_center: Option<String>,
    pub planned_work_duration: Decimal,
    pub actual_work_duration: Decimal,
    pub work_unit: String,
    pub status: String,
}
