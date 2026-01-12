use sqlx::PgPool;
use crate::domain::{ForecastPlan, ForecastPeriod};
use anyhow::Result;

pub struct ForecastRepository {
    pool: PgPool,
}

impl ForecastRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, plan: &ForecastPlan) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO forecast_plans (plan_id, plan_code, material, plant, forecast_version, model_used) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(plan.plan_id)
            .bind(&plan.plan_code)
            .bind(&plan.material)
            .bind(&plan.plant)
            .bind(&plan.forecast_version)
            .bind(&plan.model_used)
        .execute(&mut *tx).await?;

        for period in &plan.periods {
            sqlx::query(
                "INSERT INTO forecast_periods (period_id, plan_id, start_date, end_date, forecasted_quantity, unit, confidence_lower, confidence_upper) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
                .bind(period.period_id)
                .bind(period.plan_id)
                .bind(period.start_date)
                .bind(period.end_date)
                .bind(period.forecasted_quantity)
                .bind(&period.unit)
                .bind(period.confidence_lower)
                .bind(period.confidence_upper)
            .execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_code(&self, code: &str) -> Result<Option<ForecastPlan>> {
        let h = sqlx::query_as::<_, ForecastPlan>("SELECT plan_id, plan_code, material, plant, forecast_version, model_used, created_at FROM forecast_plans WHERE plan_code = $1")
            .bind(code)
            .fetch_optional(&self.pool).await?;
        if let Some(mut h) = h {
            let periods = sqlx::query_as::<_, ForecastPeriod>("SELECT * FROM forecast_periods WHERE plan_id = $1 ORDER BY start_date")
                .bind(h.plan_id)
                .fetch_all(&self.pool).await?;
            h.periods = periods;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }
}
