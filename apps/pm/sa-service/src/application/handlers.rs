use crate::application::commands::{AnalyzeSpendCommand, GetTrendCommand};
use crate::domain::{SpendDimension, TimeSeriesDataPoint};
use crate::infrastructure::repository::SpendRepository;
use anyhow::Result;
use std::sync::Arc;

pub struct SpendHandler {
    repo: Arc<SpendRepository>,
}

impl SpendHandler {
    pub fn new(repo: Arc<SpendRepository>) -> Self {
        Self { repo }
    }

    pub async fn analyze_by_category(
        &self,
        cmd: AnalyzeSpendCommand,
    ) -> Result<Vec<SpendDimension>> {
        self.repo
            .get_spend_by_category(cmd.start_date, cmd.end_date, cmd.top_n as i64)
            .await
    }

    pub async fn analyze_by_supplier(
        &self,
        cmd: AnalyzeSpendCommand,
    ) -> Result<Vec<SpendDimension>> {
        self.repo
            .get_spend_by_supplier(cmd.start_date, cmd.end_date, cmd.top_n as i64)
            .await
    }

    pub async fn get_trend(&self, cmd: GetTrendCommand) -> Result<Vec<TimeSeriesDataPoint>> {
        self.repo
            .get_spend_trend(cmd.start_date, cmd.end_date)
            .await
    }
}
