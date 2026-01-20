use crate::application::commands::{GenerateForecastCommand, TransferCommand};
use crate::application::handlers::ForecastHandler;
use crate::infrastructure::repository::ForecastRepository;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::api::proto::common::v1 as common_v1;
use crate::api::proto::sc::df::v1 as df_v1;

use df_v1::demand_forecasting_service_server::DemandForecastingService;
use df_v1::*;

pub struct DfServiceImpl {
    handler: Arc<ForecastHandler>,
    repo: Arc<ForecastRepository>,
}

impl DfServiceImpl {
    pub fn new(handler: Arc<ForecastHandler>, repo: Arc<ForecastRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl DemandForecastingService for DfServiceImpl {
    async fn generate_forecast(
        &self,
        request: Request<GenerateForecastRequest>,
    ) -> Result<Response<common_v1::JobInfo>, Status> {
        let req = request.into_inner();
        let cmd = GenerateForecastCommand {
            material: req.material,
            plant: req.plant,
        };
        let _plan_code = self
            .handler
            .generate_forecast(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(common_v1::JobInfo {
            job_id: format!("JOB{}", chrono::Utc::now().timestamp_subsec_micros()),
            job_type: "FORECAST_GENERATION".to_string(),
            status: common_v1::JobStatus::Completed as i32,
            progress_percentage: 100,
            messages: vec![],
            error_detail: "".to_string(),
            created_at: None,
            started_at: None,
            completed_at: None,
        }))
    }

    async fn get_forecast(
        &self,
        request: Request<GetForecastRequest>,
    ) -> Result<Response<ForecastPlan>, Status> {
        let req = request.into_inner();
        let plan = self
            .repo
            .find_by_code(&req.plan_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Forecast plan not found"))?;
        Ok(Response::new(ForecastPlan {
            plan_id: plan.plan_code,
            material: plan.material,
            plant: plan.plant,
            forecast_version: plan.forecast_version.unwrap_or_default(),
            periods: plan
                .periods
                .into_iter()
                .map(|p| ForecastPeriod {
                    start_date: Some(prost_types::Timestamp {
                        seconds: p
                            .start_date
                            .and_hms_opt(0, 0, 0)
                            .unwrap()
                            .and_utc()
                            .timestamp(),
                        nanos: 0,
                    }),
                    end_date: Some(prost_types::Timestamp {
                        seconds: p
                            .end_date
                            .and_hms_opt(0, 0, 0)
                            .unwrap()
                            .and_utc()
                            .timestamp(),
                        nanos: 0,
                    }),
                    forecasted_quantity: p.forecasted_quantity.map(|q| common_v1::QuantityValue {
                        value: q.to_string(),
                        unit_code: p.unit.clone(),
                    }),
                    confidence_lower_bound: p.confidence_lower.map(|l| common_v1::QuantityValue {
                        value: l.to_string(),
                        unit_code: p.unit.clone(),
                    }),
                    confidence_upper_bound: p.confidence_upper.map(|u| common_v1::QuantityValue {
                        value: u.to_string(),
                        unit_code: p.unit,
                    }),
                })
                .collect(),
            model_used: common_v1::ForecastModel::MovingAverage as i32,
            created_at: Some(prost_types::Timestamp {
                seconds: plan.created_at.timestamp(),
                nanos: 0,
            }),
        }))
    }

    async fn transfer_forecast_to_erp(
        &self,
        request: Request<TransferForecastRequest>,
    ) -> Result<Response<TransferForecastResponse>, Status> {
        let req = request.into_inner();
        let cmd = TransferCommand {
            plan_code: req.plan_id,
        };
        let success = self
            .handler
            .transfer_forecast(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(TransferForecastResponse {
            success,
            messages: vec![],
        }))
    }

    async fn update_forecast_plan(
        &self,
        _r: Request<UpdateForecastPlanRequest>,
    ) -> Result<Response<ForecastPlanResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn compare_forecast_versions(
        &self,
        _r: Request<CompareForecastVersionsRequest>,
    ) -> Result<Response<ForecastComparisonResponse>, Status> {
        Err(Status::unimplemented(""))
    }
}
