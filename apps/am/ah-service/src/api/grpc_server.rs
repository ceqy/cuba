use tonic::{Request, Response, Status};
use std::sync::Arc;
use crate::application::commands::IngestDataCommand;
use crate::application::handlers::HealthHandler;
use crate::infrastructure::repository::HealthRepository;

use crate::api::proto::am::ah::v1 as ah_v1;
use crate::api::proto::common::v1 as common_v1;

use ah_v1::intelligent_asset_health_service_server::IntelligentAssetHealthService;
use ah_v1::*;

pub struct AhServiceImpl {
    handler: Arc<HealthHandler>,
    repo: Arc<HealthRepository>,
}

impl AhServiceImpl {
    pub fn new(handler: Arc<HealthHandler>, repo: Arc<HealthRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl IntelligentAssetHealthService for AhServiceImpl {

    async fn ingest_sensor_data(
        &self,
        request: Request<IngestSensorDataRequest>,
    ) -> Result<Response<common_v1::JobInfo>, Status> {
        let req = request.into_inner();
        let cmd = IngestDataCommand {
            equipment_number: req.equipment_number,
            sensor_id: req.data_points.first().map(|d| d.sensor_id.clone()).unwrap_or_default(),
            value: req.data_points.first().map(|d| d.value.clone()).unwrap_or_default(),
        };
        let job_id = self.handler.ingest_data(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(common_v1::JobInfo {
            job_id,
            job_type: "SENSOR_INGEST".to_string(),
            status: common_v1::JobStatus::Completed as i32,
            progress_percentage: 100,
            messages: vec![],
            error_detail: "".to_string(),
            created_at: None, started_at: None, completed_at: None,
        }))
    }

    async fn get_asset_health_status(
        &self,
        request: Request<GetAssetHealthStatusRequest>,
    ) -> Result<Response<AssetHealthStatus>, Status> {
        let req = request.into_inner();
        let health = self.repo.find_health_by_equipment(&req.equipment_number).await
            .map_err(|e| Status::internal(e.to_string()))?
            .unwrap_or_else(|| crate::domain::AssetHealthStatus {
                health_id: uuid::Uuid::new_v4(),
                equipment_number: req.equipment_number.clone(),
                health_score: 85,
                status_description: Some("Good".to_string()),
                remaining_useful_life: Some("120 days".to_string()),
                last_updated: chrono::Utc::now(),
            });
        Ok(Response::new(AssetHealthStatus {
            equipment_number: health.equipment_number,
            overall_health_score: health.health_score,
            status_description: common_v1::HealthStatus::Healthy as i32,
            remaining_useful_life: health.remaining_useful_life.unwrap_or_default(),
            last_updated: Some(prost_types::Timestamp { seconds: health.last_updated.timestamp(), nanos: 0 }),
        }))
    }

    async fn get_predictive_maintenance_alerts(
        &self,
        request: Request<GetPredictiveMaintenanceAlertsRequest>,
    ) -> Result<Response<GetPredictiveMaintenanceAlertsResponse>, Status> {
        let req = request.into_inner();
        let alerts = self.repo.find_alerts_by_equipment(&req.equipment_number).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(GetPredictiveMaintenanceAlertsResponse {
            alerts: alerts.into_iter().map(|a| PredictiveMaintenanceAlert {
                alert_id: a.alert_id.to_string(),
                equipment_number: a.equipment_number,
                predicted_failure_mode: common_v1::FailureMode::Wear as i32,
                recommended_action: a.recommended_action.unwrap_or_default(),
                confidence_score: a.confidence_score.map(|c| c.to_string().parse().unwrap_or(0.0)).unwrap_or(0.85),
                alert_time: Some(prost_types::Timestamp { seconds: a.alert_time.timestamp(), nanos: 0 }),
            }).collect(),
        }))
    }

    async fn list_sensor_data(&self, _r: Request<ListSensorDataRequest>) -> Result<Response<ListSensorDataResponse>, Status> { Err(Status::unimplemented("")) }
    async fn create_health_model(&self, _r: Request<CreateHealthModelRequest>) -> Result<Response<HealthModelResponse>, Status> { Err(Status::unimplemented("")) }
}
