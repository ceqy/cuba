use tonic::{Request, Response, Status}; use std::sync::Arc;
use crate::application::handlers::IncidentHandler; use crate::infrastructure::repository::IncidentRepository;
use crate::api::proto::am::eh::v1 as eh_v1; use crate::api::proto::common::v1 as common_v1;
use eh_v1::ehs_incident_service_server::EhsIncidentService; use eh_v1::*;

pub struct EhServiceImpl { handler: Arc<IncidentHandler>, repo: Arc<IncidentRepository> }
impl EhServiceImpl { pub fn new(handler: Arc<IncidentHandler>, repo: Arc<IncidentRepository>) -> Self { Self { handler, repo } } }

#[tonic::async_trait]
impl EhsIncidentService for EhServiceImpl {
    async fn report_incident(&self, request: Request<ReportIncidentRequest>) -> Result<Response<ReportIncidentResponse>, Status> {
        let req = request.into_inner();
        let detail = req.incident.ok_or_else(|| Status::invalid_argument("Incident required"))?;
        let code = self.handler.report(detail.title, detail.description).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ReportIncidentResponse { success: true, incident_id: code, messages: vec![] }))
    }
    async fn get_incident(&self, request: Request<GetIncidentRequest>) -> Result<Response<IncidentDetail>, Status> {
        let req = request.into_inner();
        let i = self.repo.find_by_code(&req.incident_id).await.map_err(|e| Status::internal(e.to_string()))?.ok_or_else(|| Status::not_found("Not found"))?;
        Ok(Response::new(IncidentDetail { incident_id: i.incident_code, category: 0, title: i.title.unwrap_or_default(), description: i.description.unwrap_or_default(), location: i.location.unwrap_or_default(), incident_datetime: i.incident_datetime.map(|d| prost_types::Timestamp { seconds: d.timestamp(), nanos: 0 }), reported_by: i.reported_by.unwrap_or_default(), status: common_v1::IncidentStatus::Reported as i32, findings: vec![] }))
    }
    async fn update_incident(&self, _r: Request<UpdateIncidentRequest>) -> Result<Response<UpdateIncidentResponse>, Status> { Ok(Response::new(UpdateIncidentResponse { success: true, messages: vec![] })) }
    async fn close_incident(&self, _r: Request<CloseIncidentRequest>) -> Result<Response<UpdateIncidentResponse>, Status> { Ok(Response::new(UpdateIncidentResponse { success: true, messages: vec![] })) }
    async fn list_incidents(&self, _r: Request<ListIncidentsRequest>) -> Result<Response<ListIncidentsResponse>, Status> { Ok(Response::new(ListIncidentsResponse { incidents: vec![], pagination: None })) }
}
