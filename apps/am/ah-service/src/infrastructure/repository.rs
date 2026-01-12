use sqlx::PgPool;
use crate::domain::{SensorDataPoint, AssetHealthStatus, PredictiveAlert};
use anyhow::Result;

pub struct HealthRepository {
    pool: PgPool,
}

impl HealthRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_sensor_data(&self, data: &SensorDataPoint) -> Result<()> {
        sqlx::query(
            "INSERT INTO sensor_data (data_id, equipment_number, sensor_id, value, unit, recorded_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(data.data_id)
            .bind(&data.equipment_number)
            .bind(&data.sensor_id)
            .bind(&data.value)
            .bind(&data.unit)
            .bind(data.recorded_at)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn find_health_by_equipment(&self, equip: &str) -> Result<Option<AssetHealthStatus>> {
        let h = sqlx::query_as::<_, AssetHealthStatus>("SELECT health_id, equipment_number, health_score, status_description, remaining_useful_life, last_updated FROM asset_health WHERE equipment_number = $1")
            .bind(equip)
            .fetch_optional(&self.pool).await?;
        Ok(h)
    }

    pub async fn find_alerts_by_equipment(&self, equip: &str) -> Result<Vec<PredictiveAlert>> {
        let rows = sqlx::query_as::<_, PredictiveAlert>("SELECT * FROM predictive_alerts WHERE equipment_number = $1 ORDER BY alert_time DESC")
            .bind(equip)
            .fetch_all(&self.pool).await?;
        Ok(rows)
    }
}
