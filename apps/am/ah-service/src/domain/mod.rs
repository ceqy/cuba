use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorDataPoint {
    pub data_id: Uuid,
    pub equipment_number: String,
    pub sensor_id: String,
    pub value: Option<String>,
    pub unit: Option<String>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetHealthStatus {
    pub health_id: Uuid,
    pub equipment_number: String,
    pub health_score: i32,
    pub status_description: Option<String>,
    pub remaining_useful_life: Option<String>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveAlert {
    pub alert_id: Uuid,
    pub equipment_number: String,
    pub failure_mode: Option<String>,
    pub recommended_action: Option<String>,
    pub confidence_score: Option<Decimal>,
    pub alert_time: DateTime<Utc>,
}
