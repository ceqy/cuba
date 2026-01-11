use tonic::{Request, Response, Status};
use std::sync::Arc;
use crate::application::commands::ConfirmOperationCommand;
use crate::application::handlers::ProductionHandler;
use crate::infrastructure::repository::ProductionOrderRepository;

use crate::api::proto::mf::sf::v1 as sf_v1;
use crate::api::proto::common::v1 as common_v1;

use sf_v1::shop_floor_execution_service_server::ShopFloorExecutionService;
use sf_v1::*;

pub struct SfServiceImpl {
    handler: Arc<ProductionHandler>,
    repo: Arc<ProductionOrderRepository>,
}

impl SfServiceImpl {
    pub fn new(handler: Arc<ProductionHandler>, repo: Arc<ProductionOrderRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl ShopFloorExecutionService for SfServiceImpl {

    async fn get_production_order(
        &self,
        request: Request<GetProductionOrderRequest>,
    ) -> Result<Response<ProductionOrderDetail>, Status> {
        let req = request.into_inner();
        let order = self.repo.find_by_number(&req.order_number).await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Production Order not found"))?;
            
        let dt_to_ts = |d: chrono::NaiveDate| {
            let dt = d.and_hms_opt(0, 0, 0).unwrap().and_utc();
             Some(prost_types::Timestamp {
                seconds: dt.timestamp(),
                nanos: 0,
            })
        };

        Ok(Response::new(ProductionOrderDetail {
            order_number: order.order_number,
            order_type: order.order_type,
            material: order.material,
            plant: order.plant,
            total_quantity: Some(common_v1::QuantityValue {
                value: order.total_quantity.to_string(),
                unit_code: order.quantity_unit.clone(),
            }),
            delivered_quantity: Some(common_v1::QuantityValue {
                value: order.delivered_quantity.to_string(),
                unit_code: order.quantity_unit,
            }),
            basic_start_date: dt_to_ts(order.basic_start_date),
            basic_finish_date: dt_to_ts(order.basic_finish_date),
            status: common_v1::OrderStatus::Created as i32, // Simplified mapping
            operations: order.operations.into_iter().map(|op| ProductionOperation {
                operation_number: op.operation_number,
                work_center: op.work_center,
                description: op.description.unwrap_or_default(),
                confirmed_yield: Some(common_v1::QuantityValue {
                    value: op.confirmed_yield.to_string(),
                    unit_code: "PC".to_string(), // Simplified
                }),
                status: common_v1::OperationStatus::Crtd as i32,
            }).collect(),
            components: vec![], // Stub
        }))
    }

    async fn confirm_production_operation(
        &self,
        request: Request<ConfirmOperationRequest>,
    ) -> Result<Response<ConfirmOperationResponse>, Status> {
         let req = request.into_inner();
         
         let cmd = ConfirmOperationCommand {
             order_number: req.order_number,
             operation_number: req.operation_number,
             yield_quantity: req.yield_quantity.unwrap_or_default().value.parse().unwrap_or_default(),
             scrap_quantity: req.scrap_quantity.unwrap_or_default().value.parse().unwrap_or_default(),
             final_confirmation: req.final_confirmation,
             posting_date: chrono::Utc::now().date_naive(), // Ignore req date for MVP safety
         };
         
         let conf_num = self.handler.confirm_operation(cmd).await
             .map_err(|e| Status::internal(e.to_string()))?;
             
         Ok(Response::new(ConfirmOperationResponse {
             success: true,
             confirmation_number: conf_num,
             messages: vec![],
         }))
    }

    // Stubs
    async fn cancel_confirmation(&self, _r: Request<CancelConfirmationRequest>) -> Result<Response<CancelConfirmationResponse>, Status> { Err(Status::unimplemented("")) }
    async fn post_material_consumption(&self, _r: Request<PostMaterialMovementRequest>) -> Result<Response<common_v1::StockMovementResponse>, Status> { Err(Status::unimplemented("")) }
    async fn post_goods_receipt_for_order(&self, _r: Request<PostMaterialMovementRequest>) -> Result<Response<common_v1::StockMovementResponse>, Status> { Err(Status::unimplemented("")) }
}
