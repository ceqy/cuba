use tonic::{Request, Response, Status};
use std::sync::Arc;
use crate::application::commands::{CreateNotificationCommand, CreateOrderCommand, ConfirmOperationCommand};
use crate::application::handlers::MaintenanceHandler;
use crate::infrastructure::repository::MaintenanceRepository;

use crate::api::proto::am::pm::v1 as pm_v1;
use crate::api::proto::common::v1 as common_v1;

use pm_v1::asset_maintenance_service_server::AssetMaintenanceService;
use pm_v1::*;

pub struct PmServiceImpl {
    handler: Arc<MaintenanceHandler>,
    repo: Arc<MaintenanceRepository>,
}

impl PmServiceImpl {
    pub fn new(handler: Arc<MaintenanceHandler>, repo: Arc<MaintenanceRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl AssetMaintenanceService for PmServiceImpl {

    async fn create_maintenance_notification(
        &self,
        request: Request<CreateNotificationRequest>,
    ) -> Result<Response<NotificationResponse>, Status> {
        let req = request.into_inner();
        let notif = req.notification.unwrap_or_default();
        let cmd = CreateNotificationCommand {
            notification_type: notif.notification_type,
            description: notif.description,
            equipment_number: notif.equipment_number,
        };
        let num = self.handler.create_notification(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(NotificationResponse {
            success: true,
            notification_number: num,
            messages: vec![],
        }))
    }

    async fn create_maintenance_order(
        &self,
        request: Request<CreateMaintenanceOrderRequest>,
    ) -> Result<Response<MaintenanceOrderResponse>, Status> {
        let req = request.into_inner();
        let cmd = CreateOrderCommand {
            order_type: req.order_type,
            description: req.description,
            equipment_number: req.equipment_number,
            maintenance_plant: req.planning_plant, // Simplified mapping
        };
        let num = self.handler.create_order(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(MaintenanceOrderResponse {
            success: true,
            order_number: num,
            status: common_v1::OrderStatus::Created as i32,
            messages: vec![],
        }))
    }

    async fn get_maintenance_order(
        &self,
        request: Request<GetMaintenanceOrderRequest>,
    ) -> Result<Response<MaintenanceOrderDetail>, Status> {
        let req = request.into_inner();
        let order = self.repo.find_order_by_number(&req.order_number).await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Order not found"))?;
        let maint_plant = order.maintenance_plant.clone(); // Clone before move
        Ok(Response::new(MaintenanceOrderDetail {
            order_number: order.order_number,
            order_type: order.order_type,
            description: order.description.unwrap_or_default(),
            notification_number: order.notification_number.unwrap_or_default(),
            equipment_number: order.equipment_number.unwrap_or_default(),
            functional_location: order.functional_location.unwrap_or_default(),
            maintenance_plant: order.maintenance_plant,
            planning_plant: order.planning_plant.unwrap_or_default(),
            planner_group: "".to_string(),
            main_work_center: order.main_work_center.unwrap_or_default(),
            system_status: order.system_status,
            user_status: "".to_string(),
            priority: order.priority,
            basic_start_date: None, basic_finish_date: None,
            scheduled_start_date: None, scheduled_finish_date: None,
            actual_start_date: None, actual_finish_date: None,
            currency: "CNY".to_string(),
            estimated_total_cost: None, actual_total_cost: None,
            operations: order.operations.into_iter().map(|op| MaintenanceOperation {
                operation_number: op.operation_number,
                sub_operation_number: "".to_string(),
                control_key: "".to_string(),
                work_center: op.work_center.unwrap_or_default(),
                plant: maint_plant.clone(),
                description: op.description.unwrap_or_default(),
                long_description: "".to_string(),
                planned_work_duration: op.planned_work_duration.to_string(),
                actual_work_duration: op.actual_work_duration.to_string(),
                work_unit: op.work_unit,
                quantity: None,
                system_status: common_v1::OperationStatus::Crtd as i32,
            }).collect(),
            components: vec![],
        }))
    }

    async fn confirm_maintenance_operation(
        &self,
        request: Request<ConfirmMaintenanceOperationRequest>,
    ) -> Result<Response<ConfirmMaintenanceOperationResponse>, Status> {
        let req = request.into_inner();
        let cmd = ConfirmOperationCommand {
            order_number: req.order_number,
            operation_number: req.operation_number,
            actual_duration: req.actual_work_duration.parse().unwrap_or_default(),
        };
        let conf_num = self.handler.confirm_operation(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ConfirmMaintenanceOperationResponse {
            success: true,
            confirmation_number: conf_num,
            messages: vec![],
        }))
    }

    // Stubs
    async fn update_maintenance_notification(&self, _r: Request<UpdateNotificationRequest>) -> Result<Response<NotificationResponse>, Status> { Err(Status::unimplemented("")) }
    async fn release_maintenance_notification(&self, _r: Request<ReleaseNotificationRequest>) -> Result<Response<NotificationResponse>, Status> { Err(Status::unimplemented("")) }
    async fn list_maintenance_notifications(&self, _r: Request<ListNotificationsRequest>) -> Result<Response<ListNotificationsResponse>, Status> { Err(Status::unimplemented("")) }
    async fn update_maintenance_order(&self, _r: Request<UpdateMaintenanceOrderRequest>) -> Result<Response<MaintenanceOrderResponse>, Status> { Err(Status::unimplemented("")) }
    async fn release_maintenance_order(&self, _r: Request<ReleaseMaintenanceOrderRequest>) -> Result<Response<MaintenanceOrderResponse>, Status> { Err(Status::unimplemented("")) }
    async fn cancel_maintenance_order(&self, _r: Request<CancelMaintenanceOrderRequest>) -> Result<Response<MaintenanceOrderResponse>, Status> { Err(Status::unimplemented("")) }
    async fn complete_maintenance_order(&self, _r: Request<CompleteMaintenanceOrderRequest>) -> Result<Response<MaintenanceOrderResponse>, Status> { Err(Status::unimplemented("")) }
    async fn list_maintenance_orders(&self, _r: Request<ListOrdersRequest>) -> Result<Response<ListOrdersResponse>, Status> { Err(Status::unimplemented("")) }
    async fn get_order_components(&self, _r: Request<GetOrderComponentsRequest>) -> Result<Response<OrderComponentsResponse>, Status> { Err(Status::unimplemented("")) }
}
