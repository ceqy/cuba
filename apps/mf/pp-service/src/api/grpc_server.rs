use tonic::{Request, Response, Status};
use std::sync::Arc;
use crate::application::commands::RunMrpCommand;
use crate::application::handlers::RunMrpHandler;
use crate::infrastructure::repository::PlannedOrderRepository;

use crate::api::proto::mf::pp::v1 as pp_v1;
use crate::api::proto::common::v1 as common_v1;

use pp_v1::production_planning_service_server::ProductionPlanningService;
use pp_v1::*;

pub struct PpServiceImpl {
    mrp_handler: Arc<RunMrpHandler>,
    repo: Arc<PlannedOrderRepository>,
}

impl PpServiceImpl {
    pub fn new(mrp_handler: Arc<RunMrpHandler>, repo: Arc<PlannedOrderRepository>) -> Self {
        Self { mrp_handler, repo }
    }
}

#[tonic::async_trait]
impl ProductionPlanningService for PpServiceImpl {

    async fn run_mrp(
        &self,
        request: Request<RunMrpRequest>,
    ) -> Result<Response<common_v1::JobInfo>, Status> {
        let req = request.into_inner();
        
        let cmd = RunMrpCommand {
            plant: req.plant,
            materials: req.materials,
            run_type: req.run_type,
            planning_mode: req.planning_mode,
        };

        let job_id = self.mrp_handler.handle(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(common_v1::JobInfo {
            job_id,
            job_type: "MRP".to_string(),
            status: common_v1::JobStatus::Completed as i32,
            ..Default::default()
        }))
    }
    
    async fn list_planned_orders(
        &self,
        request: Request<ListPlannedOrdersRequest>,
    ) -> Result<Response<ListPlannedOrdersResponse>, Status> {
        let req = request.into_inner();
        let orders = self.repo.list_by_plant(&req.plant).await
            .map_err(|e| Status::internal(e.to_string()))?;
            
        let dt_to_ts = |d: chrono::NaiveDate| {
            let dt = d.and_hms_opt(0, 0, 0).unwrap().and_utc();
             Some(prost_types::Timestamp {
                seconds: dt.timestamp(),
                nanos: 0,
            })
        };

        Ok(Response::new(ListPlannedOrdersResponse {
            planned_orders: orders.into_iter().map(|o| PlannedOrder {
                planned_order_number: o.planned_order_number,
                material: o.material,
                plant: o.plant,
                order_quantity: Some(common_v1::QuantityValue {
                    value: o.order_quantity.to_string(),
                    unit_code: o.quantity_unit,
                }),
                order_start_date: dt_to_ts(o.order_start_date),
                order_finish_date: dt_to_ts(o.order_finish_date),
                mrp_controller: o.mrp_controller.unwrap_or_default(),
                conversion_indicator: o.conversion_indicator,
            }).collect(),
            pagination: None,
        }))
    }
    
    // Stubs
    async fn simulate_mrp(&self, _r: Request<RunMrpRequest>) -> Result<Response<common_v1::JobInfo>, Status> { Err(Status::unimplemented("")) }
    async fn get_mrp_status(&self, _r: Request<GetMrpStatusRequest>) -> Result<Response<common_v1::JobInfo>, Status> { Err(Status::unimplemented("")) }
    async fn convert_planned_order(&self, _r: Request<ConvertPlannedOrderRequest>) -> Result<Response<ConvertPlannedOrderResponse>, Status> { Err(Status::unimplemented("")) }
    async fn get_work_center_capacity_load(&self, _r: Request<GetWorkCenterCapacityLoadRequest>) -> Result<Response<GetWorkCenterCapacityLoadResponse>, Status> { Err(Status::unimplemented("")) }

}
