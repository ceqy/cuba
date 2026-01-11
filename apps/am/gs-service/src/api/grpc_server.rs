use tonic::{Request, Response, Status}; use std::sync::Arc;
use crate::application::handlers::SettingsHandler; use crate::infrastructure::repository::SettingsRepository;
use crate::api::proto::am::gs::v1 as gs_v1; use crate::api::proto::common::v1 as common_v1;
use gs_v1::geo_service_server::GeoService; use gs_v1::*;

pub struct GsServiceImpl { handler: Arc<SettingsHandler>, _repo: Arc<SettingsRepository> }
impl GsServiceImpl { pub fn new(handler: Arc<SettingsHandler>, repo: Arc<SettingsRepository>) -> Self { Self { handler, _repo: repo } } }

#[tonic::async_trait]
impl GeoService for GsServiceImpl {
    async fn get_asset_location(&self, request: Request<GetAssetLocationRequest>) -> Result<Response<AssetLocation>, Status> {
        let req = request.into_inner();
        Ok(Response::new(AssetLocation { asset_id: req.asset_id, location: Some(GeoPoint { latitude: 31.2304, longitude: 121.4737 }), timestamp: Some(prost_types::Timestamp { seconds: chrono::Utc::now().timestamp(), nanos: 0 }) }))
    }
    async fn update_asset_location(&self, request: Request<UpdateAssetLocationRequest>) -> Result<Response<UpdateLocationResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(UpdateLocationResponse { success: true, message: None }))
    }
    async fn check_geofence(&self, _r: Request<CheckGeofenceRequest>) -> Result<Response<CheckGeofenceResponse>, Status> {
        Ok(Response::new(CheckGeofenceResponse { is_inside: true }))
    }
    async fn create_geofence(&self, _r: Request<CreateGeofenceRequest>) -> Result<Response<GeofenceResponse>, Status> {
        Ok(Response::new(GeofenceResponse { success: true, fence_id: format!("GEO{}", chrono::Utc::now().timestamp_subsec_micros()) }))
    }
    async fn calculate_route(&self, _r: Request<CalculateRouteRequest>) -> Result<Response<common_v1::JobInfo>, Status> {
        Ok(Response::new(common_v1::JobInfo { job_id: format!("ROUTE{}", chrono::Utc::now().timestamp_subsec_micros()), job_type: "ROUTE_CALC".to_string(), status: common_v1::JobStatus::Completed as i32, progress_percentage: 100, messages: vec![], error_detail: "".to_string(), created_at: None, started_at: None, completed_at: None }))
    }
    async fn get_route_result(&self, _r: Request<GetRouteResultRequest>) -> Result<Response<Route>, Status> {
        Ok(Response::new(Route { distance_meters: 1000.0, duration_seconds: 600, steps: vec![] }))
    }
}
