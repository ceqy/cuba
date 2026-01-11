use tonic::{Request, Response, Status};
use std::sync::Arc;
use crate::application::commands::{AnalyzeSalesCommand, GetTrendCommand};
use crate::application::handlers::SalesHandler;

use crate::api::proto::sd::an::v1 as an_v1;
use crate::api::proto::common::v1 as common_v1;

use an_v1::sales_analytics_service_server::SalesAnalyticsService;
use an_v1::*;

pub struct AnServiceImpl {
    handler: Arc<SalesHandler>,
}

impl AnServiceImpl {
    pub fn new(handler: Arc<SalesHandler>) -> Self {
        Self { handler }
    }
}

#[tonic::async_trait]
impl SalesAnalyticsService for AnServiceImpl {

    async fn get_sales_by_customer(
        &self,
        request: Request<SalesAnalysisRequest>,
    ) -> Result<Response<SalesAnalysisResponse>, Status> {
        let req = request.into_inner();
        let cmd = AnalyzeSalesCommand {
            start_date: chrono::Utc::now().date_naive() - chrono::Duration::days(365),
            end_date: chrono::Utc::now().date_naive(),
            top_n: req.top_n,
        };
        let results = self.handler.analyze_by_customer(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(SalesAnalysisResponse {
            results: results.into_iter().map(|r| SalesDimension {
                id: r.id,
                name: r.name,
                revenue: Some(common_v1::MonetaryValue { value: r.revenue.to_string(), currency_code: r.currency }),
                quantity_sold: r.quantity_sold.to_string(),
                unit: r.unit,
            }).collect(),
        }))
    }

    async fn get_sales_by_product(
        &self,
        request: Request<SalesAnalysisRequest>,
    ) -> Result<Response<SalesAnalysisResponse>, Status> {
        let req = request.into_inner();
        let cmd = AnalyzeSalesCommand {
            start_date: chrono::Utc::now().date_naive() - chrono::Duration::days(365),
            end_date: chrono::Utc::now().date_naive(),
            top_n: req.top_n,
        };
        let results = self.handler.analyze_by_product(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(SalesAnalysisResponse {
            results: results.into_iter().map(|r| SalesDimension {
                id: r.id,
                name: r.name,
                revenue: Some(common_v1::MonetaryValue { value: r.revenue.to_string(), currency_code: r.currency }),
                quantity_sold: r.quantity_sold.to_string(),
                unit: r.unit,
            }).collect(),
        }))
    }

    async fn get_sales_trend(
        &self,
        request: Request<SalesTrendRequest>,
    ) -> Result<Response<SalesTrendResponse>, Status> {
        let req = request.into_inner();
        let cmd = GetTrendCommand {
            start_date: chrono::Utc::now().date_naive() - chrono::Duration::days(365),
            end_date: chrono::Utc::now().date_naive(),
        };
        let trend = self.handler.get_trend(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(SalesTrendResponse {
            trend: trend.into_iter().map(|t| TimeSeriesDataPoint {
                period: t.period,
                revenue: Some(common_v1::MonetaryValue { value: t.revenue.to_string(), currency_code: t.currency }),
            }).collect(),
        }))
    }

    async fn export_sales_report(&self, _r: Request<ExportSalesReportRequest>) -> Result<Response<common_v1::JobInfo>, Status> {
        Ok(Response::new(common_v1::JobInfo {
            job_id: format!("EXP{}", chrono::Utc::now().timestamp_subsec_micros()),
            job_type: "EXPORT".to_string(),
            status: common_v1::JobStatus::Running as i32,
            progress_percentage: 0,
            messages: vec![],
            error_detail: "".to_string(),
            created_at: None,
            started_at: None,
            completed_at: None,
        }))
    }
}
