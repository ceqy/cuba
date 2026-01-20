use crate::application::commands::{GenerateForecastCommand, TransferCommand};
use crate::domain::{ForecastPeriod, ForecastPlan};
use crate::infrastructure::repository::ForecastRepository;
use anyhow::Result;
use chrono::Utc;
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

pub struct ForecastHandler {
    repo: Arc<ForecastRepository>,
}

impl ForecastHandler {
    pub fn new(repo: Arc<ForecastRepository>) -> Self {
        Self { repo }
    }

    pub async fn generate_forecast(&self, cmd: GenerateForecastCommand) -> Result<String> {
        let plan_id = Uuid::new_v4();
        let plan_code = format!("FC{}", Utc::now().timestamp_subsec_micros());

        // Simplified: generate 3 monthly periods
        let mut periods = Vec::new();
        for i in 0..3 {
            periods.push(ForecastPeriod {
                period_id: Uuid::new_v4(),
                plan_id,
                start_date: (Utc::now() + chrono::Duration::days((i * 30) as i64)).date_naive(),
                end_date: (Utc::now() + chrono::Duration::days(((i + 1) * 30) as i64)).date_naive(),
                forecasted_quantity: Some(Decimal::new(100 + (i * 10), 0)),
                unit: "EA".to_string(),
                confidence_lower: Some(Decimal::new(90 + (i * 10), 0)),
                confidence_upper: Some(Decimal::new(110 + (i * 10), 0)),
            });
        }

        let plan = ForecastPlan {
            plan_id,
            plan_code: plan_code.clone(),
            material: cmd.material,
            plant: cmd.plant,
            forecast_version: Some("V1".to_string()),
            model_used: Some("MOVING_AVERAGE".to_string()),
            created_at: Utc::now(),
            periods,
        };

        self.repo.save(&plan).await?;
        Ok(plan_code)
    }

    pub async fn transfer_forecast(&self, _cmd: TransferCommand) -> Result<bool> {
        Ok(true)
    }
}
