use tonic::{Request, Response, Status};
use std::sync::Arc;
use crate::application::commands::{AssignTechnicianCommand, UpdateStatusCommand};
use crate::application::handlers::ServiceHandler;
use crate::infrastructure::repository::ServiceOrderRepository;

use crate::api::proto::cs::fd::v1 as fd_v1;
use crate::api::proto::common::v1 as common_v1;

use fd_v1::field_service_dispatch_service_server::FieldServiceDispatchService;
use fd_v1::*;

pub struct FdServiceImpl {
    handler: Arc<ServiceHandler>,
    repo: Arc<ServiceOrderRepository>,
}

impl FdServiceImpl {
    pub fn new(handler: Arc<ServiceHandler>, repo: Arc<ServiceOrderRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl FieldServiceDispatchService for FdServiceImpl {

    async fn get_service_order(
        &self,
        request: Request<GetServiceOrderRequest>,
    ) -> Result<Response<ServiceOrderDetail>, Status> {
        let req = request.into_inner();
        let order = self.repo.find_by_number(&req.order_number).await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Order not found"))?;
        Ok(Response::new(ServiceOrderDetail {
            order_number: order.order_number,
            order_type: common_v1::ServiceOrderType::Repair as i32,
            customer_id: order.customer_id,
            service_location: None,
            description: order.description.unwrap_or_default(),
            planned_start_datetime: None,
            assigned_technician_id: order.assigned_technician_id.unwrap_or_default(),
            status: common_v1::ServiceOrderStatus::Created as i32,
            audit_data: None,
        }))
    }

    async fn assign_technician(
        &self,
        request: Request<AssignTechnicianRequest>,
    ) -> Result<Response<AssignTechnicianResponse>, Status> {
        let req = request.into_inner();
        let cmd = AssignTechnicianCommand {
            order_number: req.order_number,
            technician_id: req.technician_id,
            scheduled_time: chrono::Utc::now(), // Simplified
        };
        self.handler.assign_technician(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(AssignTechnicianResponse {
            success: true,
            messages: vec![],
        }))
    }

    async fn update_service_order_status(
        &self,
        request: Request<UpdateServiceOrderStatusRequest>,
    ) -> Result<Response<UpdateServiceOrderStatusResponse>, Status> {
        let req = request.into_inner();
        let status_str = match req.new_status {
            1 => "OPEN",
            2 => "ASSIGNED",
            3 => "IN_PROGRESS",
            4 => "COMPLETED",
            5 => "CANCELLED",
            _ => "OPEN",
        };
        let cmd = UpdateStatusCommand {
            order_number: req.order_number,
            new_status: status_str.to_string(),
        };
        self.handler.update_status(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(UpdateServiceOrderStatusResponse {
            success: true,
            messages: vec![],
        }))
    }

    // Stubs
    async fn list_service_orders(&self, _r: Request<ListServiceOrdersRequest>) -> Result<Response<ListServiceOrdersResponse>, Status> { Err(Status::unimplemented("")) }
    async fn cancel_assignment(&self, _r: Request<CancelAssignmentRequest>) -> Result<Response<AssignTechnicianResponse>, Status> { Err(Status::unimplemented("")) }
    async fn reschedule_order(&self, _r: Request<RescheduleOrderRequest>) -> Result<Response<ServiceOrderDetail>, Status> { Err(Status::unimplemented("")) }
}
