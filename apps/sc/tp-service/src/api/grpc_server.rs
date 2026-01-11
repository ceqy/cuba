use tonic::{Request, Response, Status};
use std::sync::Arc;
use crate::application::commands::{CreateShipmentCommand, UpdateStatusCommand};
use crate::application::handlers::ShipmentHandler;
use crate::infrastructure::repository::ShipmentRepository;
use crate::api::proto::sc::tp::v1 as tp_v1;
use crate::api::proto::common::v1 as common_v1;
use tp_v1::transportation_planning_service_server::TransportationPlanningService;
use tp_v1::*;

pub struct TpServiceImpl { handler: Arc<ShipmentHandler>, repo: Arc<ShipmentRepository> }

impl TpServiceImpl {
    pub fn new(handler: Arc<ShipmentHandler>, repo: Arc<ShipmentRepository>) -> Self { Self { handler, repo } }
}

#[tonic::async_trait]
impl TransportationPlanningService for TpServiceImpl {
    async fn create_shipment(&self, request: Request<CreateShipmentRequest>) -> Result<Response<common_v1::JobInfo>, Status> {
        let req = request.into_inner();
        let cmd = CreateShipmentCommand { delivery_numbers: req.delivery_numbers, planning_point: req.transportation_planning_point };
        let _num = self.handler.create_shipment(cmd).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(common_v1::JobInfo { job_id: format!("JOB{}", chrono::Utc::now().timestamp_subsec_micros()), job_type: "CREATE_SHIPMENT".to_string(), status: common_v1::JobStatus::Completed as i32, progress_percentage: 100, messages: vec![], error_detail: "".to_string(), created_at: None, started_at: None, completed_at: None }))
    }

    async fn get_shipment(&self, request: Request<GetShipmentRequest>) -> Result<Response<ShipmentDetail>, Status> {
        let req = request.into_inner();
        let s = self.repo.find_by_number(&req.shipment_number).await.map_err(|e| Status::internal(e.to_string()))?.ok_or_else(|| Status::not_found("Shipment not found"))?;
        Ok(Response::new(ShipmentDetail {
            shipment_number: s.shipment_number,
            header: Some(ShipmentHeader { shipment_type: 0, transportation_planning_point: s.transportation_planning_point.unwrap_or_default(), carrier: s.carrier.unwrap_or_default(), overall_status: 0, planned_departure_date: s.planned_departure.map(|d| prost_types::Timestamp { seconds: d.timestamp(), nanos: 0 }), planned_arrival_date: s.planned_arrival.map(|d| prost_types::Timestamp { seconds: d.timestamp(), nanos: 0 }) }),
            items: s.items.into_iter().map(|i| ShipmentItem { item_number: i.item_number, delivery_number: i.delivery_number, total_weight: i.total_weight.map(|w| common_v1::QuantityValue { value: w.to_string(), unit_code: i.weight_unit }), volume: i.volume.map(|v| common_v1::QuantityValue { value: v.to_string(), unit_code: i.volume_unit }) }).collect(),
            audit_data: None,
        }))
    }

    async fn update_shipment_status(&self, request: Request<UpdateShipmentStatusRequest>) -> Result<Response<ShipmentResponse>, Status> {
        let req = request.into_inner();
        let cmd = UpdateStatusCommand { shipment_number: req.shipment_number.clone(), new_status: format!("{:?}", req.new_status) };
        let _ = self.handler.update_status(cmd).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ShipmentResponse { success: true, shipment_number: req.shipment_number, messages: vec![] }))
    }

    async fn cancel_shipment(&self, _r: Request<CancelShipmentRequest>) -> Result<Response<ShipmentResponse>, Status> { Err(Status::unimplemented("")) }
    async fn assign_carrier(&self, _r: Request<AssignCarrierRequest>) -> Result<Response<ShipmentResponse>, Status> { Err(Status::unimplemented("")) }
    async fn list_shipments(&self, _r: Request<ListShipmentsRequest>) -> Result<Response<ListShipmentsResponse>, Status> { Err(Status::unimplemented("")) }
}
