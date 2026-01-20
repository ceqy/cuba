use crate::application::commands::{AnalyzeSalesCommand, GetTrendCommand};
use crate::domain::{SalesDimension, TimeSeriesDataPoint};
use crate::infrastructure::repository::SalesRepository;
use anyhow::Result;
use std::sync::Arc;

pub struct SalesHandler {
    repo: Arc<SalesRepository>,
}

impl SalesHandler {
    pub fn new(repo: Arc<SalesRepository>) -> Self {
        Self { repo }
    }

    pub async fn analyze_by_customer(
        &self,
        cmd: AnalyzeSalesCommand,
    ) -> Result<Vec<SalesDimension>> {
        self.repo
            .get_sales_by_customer(cmd.start_date, cmd.end_date, cmd.top_n as i64)
            .await
    }

    pub async fn analyze_by_product(
        &self,
        cmd: AnalyzeSalesCommand,
    ) -> Result<Vec<SalesDimension>> {
        self.repo
            .get_sales_by_product(cmd.start_date, cmd.end_date, cmd.top_n as i64)
            .await
    }

    pub async fn get_trend(&self, cmd: GetTrendCommand) -> Result<Vec<TimeSeriesDataPoint>> {
        self.repo
            .get_sales_trend(cmd.start_date, cmd.end_date)
            .await
    }
}
