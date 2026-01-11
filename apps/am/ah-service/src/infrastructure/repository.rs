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
        sqlx::query!(
            "INSERT INTO sensor_data (data_id, equipment_number, sensor_id, value, unit, recorded_at) VALUES ($1, $2, $3, $4, $5, $6)",
            data.data_id, data.equipment_number, data.sensor_id, data.value, data.unit, data.recorded_at
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn find_health_by_equipment(&self, equip: &str) -> Result<Option<AssetHealthStatus>> {
        let h = sqlx::query!("SELECT * FROM asset_health WHERE equipment_number = $1", equip)
            .fetch_optional(&self.pool).await?;
        Ok(h.map(|r| AssetHealthStatus {
            health_id: r.health_id,
            equipment_number: r.equipment_number,
            health_score: r.health_score.unwrap_or(100),
            status_description: r.status_description,
            remaining_useful_life: r.remaining_useful_life,
            last_updated: r.last_updated,
        }))
    }

    pub async fn find_alerts_by_equipment(&self, equip: &str) -> Result<Vec<PredictiveAlert>> {
        let rows = sqlx::query!("SELECT * FROM predictive_alerts WHERE equipment_number = $1 ORDER BY alert_time DESC", equip)
            .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| PredictiveAlert {
            alert_id: r.alert_id,
            equipment_number: r.equipment_number,
            failure_mode: r.failure_mode,
            recommended_action: r.recommended_action,
            confidence_score: r.confidence_score,
            alert_time: r.alert_time,
        }).collect())
    }
}
