use crate::application::commands::{AnalyzeSpendCommand, GetTrendCommand};
use crate::application::handlers::SpendHandler;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::api::proto::common::v1 as common_v1;
use crate::api::proto::pm::sa::v1 as sa_v1;

use sa_v1::spend_analytics_service_server::SpendAnalyticsService;
use sa_v1::*;

pub struct SaServiceImpl {
    handler: Arc<SpendHandler>,
}

impl SaServiceImpl {
    pub fn new(handler: Arc<SpendHandler>) -> Self {
        Self { handler }
    }
}

#[tonic::async_trait]
impl SpendAnalyticsService for SaServiceImpl {
    async fn get_spend_by_category(
        &self,
        request: Request<SpendAnalysisRequest>,
    ) -> Result<Response<SpendAnalysisResponse>, Status> {
        let req = request.into_inner();
        let _time_range = req.time_range.unwrap_or_default();
        let cmd = AnalyzeSpendCommand {
            start_date: chrono::Utc::now().date_naive() - chrono::Duration::days(365),
            end_date: chrono::Utc::now().date_naive(),
            top_n: req.top_n,
        };
        let results = self
            .handler
            .analyze_by_category(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(SpendAnalysisResponse {
            results: results
                .into_iter()
                .map(|r| SpendDimension {
                    id: r.id,
                    name: r.name,
                    spend_amount: Some(common_v1::MonetaryValue {
                        value: r.spend_amount.to_string(),
                        currency_code: r.currency,
                    }),
                    document_count: r.document_count,
                })
                .collect(),
        }))
    }

    async fn get_spend_by_supplier(
        &self,
        request: Request<SpendAnalysisRequest>,
    ) -> Result<Response<SpendAnalysisResponse>, Status> {
        let req = request.into_inner();
        let cmd = AnalyzeSpendCommand {
            start_date: chrono::Utc::now().date_naive() - chrono::Duration::days(365),
            end_date: chrono::Utc::now().date_naive(),
            top_n: req.top_n,
        };
        let results = self
            .handler
            .analyze_by_supplier(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(SpendAnalysisResponse {
            results: results
                .into_iter()
                .map(|r| SpendDimension {
                    id: r.id,
                    name: r.name,
                    spend_amount: Some(common_v1::MonetaryValue {
                        value: r.spend_amount.to_string(),
                        currency_code: r.currency,
                    }),
                    document_count: r.document_count,
                })
                .collect(),
        }))
    }

    async fn get_spend_trend(
        &self,
        request: Request<SpendTrendRequest>,
    ) -> Result<Response<SpendTrendResponse>, Status> {
        let _req = request.into_inner();
        let cmd = GetTrendCommand {
            start_date: chrono::Utc::now().date_naive() - chrono::Duration::days(365),
            end_date: chrono::Utc::now().date_naive(),
        };
        let trend = self
            .handler
            .get_trend(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(SpendTrendResponse {
            trend: trend
                .into_iter()
                .map(|t| TimeSeriesDataPoint {
                    period: t.period,
                    spend_amount: Some(common_v1::MonetaryValue {
                        value: t.spend_amount.to_string(),
                        currency_code: t.currency,
                    }),
                })
                .collect(),
        }))
    }
}
