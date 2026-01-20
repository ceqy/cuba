use crate::application::commands::{ChangeStatusCommand, CreateCycleCommand};
use crate::application::handlers::KanbanHandler;
use crate::infrastructure::repository::KanbanRepository;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::api::proto::common::v1 as common_v1;
use crate::api::proto::mf::kb::v1 as kb_v1;

use kb_v1::kanban_service_server::KanbanService;
use kb_v1::*;

pub struct KbServiceImpl {
    handler: Arc<KanbanHandler>,
    repo: Arc<KanbanRepository>,
}

impl KbServiceImpl {
    pub fn new(handler: Arc<KanbanHandler>, repo: Arc<KanbanRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl KanbanService for KbServiceImpl {
    async fn create_kanban_control_cycle(
        &self,
        request: Request<CreateControlCycleRequest>,
    ) -> Result<Response<ControlCycleResponse>, Status> {
        let req = request.into_inner();
        let cycle = req.control_cycle.unwrap_or_default();
        let qty = cycle
            .quantity_per_kanban
            .map(|q| Decimal::from_str(&q.value).unwrap_or_default())
            .unwrap_or_else(|| Decimal::new(100, 0));
        let cmd = CreateCycleCommand {
            material: cycle.material,
            plant: cycle.plant,
            supply_area: if cycle.production_supply_area.is_empty() {
                None
            } else {
                Some(cycle.production_supply_area)
            },
            number_of_kanbans: cycle.number_of_kanbans,
            qty_per_kanban: qty,
        };
        let num = self
            .handler
            .create_cycle(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ControlCycleResponse {
            success: true,
            control_cycle_number: num,
            messages: vec![],
        }))
    }

    async fn get_kanban_control_cycle(
        &self,
        request: Request<GetKanbanControlCycleRequest>,
    ) -> Result<Response<KanbanControlCycle>, Status> {
        let req = request.into_inner();
        let c = self
            .repo
            .find_cycle_by_number(&req.control_cycle_number)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Cycle not found"))?;
        Ok(Response::new(KanbanControlCycle {
            control_cycle_number: c.cycle_number,
            material: c.material,
            plant: c.plant,
            production_supply_area: c.supply_area.unwrap_or_default(),
            number_of_kanbans: c.number_of_kanbans,
            quantity_per_kanban: Some(common_v1::QuantityValue {
                value: c.qty_per_kanban.to_string(),
                unit_code: c.unit,
            }),
            replenishment_strategy: common_v1::ReplenishmentStrategy::Unspecified as i32,
        }))
    }

    async fn change_kanban_status(
        &self,
        request: Request<ChangeKanbanStatusRequest>,
    ) -> Result<Response<ChangeKanbanStatusResponse>, Status> {
        let req = request.into_inner();
        let new_status = match req.new_status {
            1 => "FULL",
            2 => "EMPTY",
            3 => "IN_TRANSIT",
            _ => "UNKNOWN",
        };
        let cmd = ChangeStatusCommand {
            container_code: req.container_id,
            new_status: new_status.to_string(),
        };
        let replenishment_doc = self
            .handler
            .change_status(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ChangeKanbanStatusResponse {
            success: true,
            replenishment_document_number: replenishment_doc.unwrap_or_default(),
            messages: vec![],
        }))
    }

    async fn list_kanban_containers(
        &self,
        request: Request<ListKanbanContainersRequest>,
    ) -> Result<Response<ListKanbanContainersResponse>, Status> {
        let req = request.into_inner();
        let cycle = self
            .repo
            .find_cycle_by_number(&req.material)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        let containers = if let Some(c) = cycle {
            self.repo
                .list_containers_by_cycle(c.cycle_id)
                .await
                .map_err(|e| Status::internal(e.to_string()))?
        } else {
            vec![]
        };
        Ok(Response::new(ListKanbanContainersResponse {
            containers: containers
                .into_iter()
                .map(|k| KanbanContainer {
                    container_id: k.container_code,
                    control_cycle_number: "".to_string(),
                    status: common_v1::KanbanStatus::Full as i32,
                })
                .collect(),
            pagination: None,
        }))
    }

    async fn update_kanban_control_cycle(
        &self,
        _r: Request<UpdateControlCycleRequest>,
    ) -> Result<Response<ControlCycleResponse>, Status> {
        Err(Status::unimplemented(""))
    }
}
