use crate::application::commands::ExecuteAllocationCommand;
use crate::application::handlers::AllocationHandler;
use crate::infrastructure::repository::AllocationRepository;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::api::proto::common::v1 as common_v1;
use crate::api::proto::fi::co::v1 as co_v1;

use co_v1::controlling_allocation_service_server::ControllingAllocationService;
use co_v1::*;

pub struct CoServiceImpl {
    handler: Arc<AllocationHandler>,
    repo: Arc<AllocationRepository>,
}

impl CoServiceImpl {
    pub fn new(handler: Arc<AllocationHandler>, repo: Arc<AllocationRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl ControllingAllocationService for CoServiceImpl {
    async fn execute_cost_center_allocation(
        &self,
        request: Request<ExecuteAllocationRequest>,
    ) -> Result<Response<AllocationResponse>, Status> {
        let req = request.into_inner();
        let cmd = ExecuteAllocationCommand {
            controlling_area: req.controlling_area,
            fiscal_year: req.fiscal_year,
            fiscal_period: req.fiscal_period,
            allocation_cycle: req.allocation_cycle,
            allocation_type: "COST_CENTER".to_string(),
            test_run: req.test_run,
        };
        let run_id = self
            .handler
            .execute_allocation(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(AllocationResponse {
            success: true,
            run_id,
            messages: vec![],
        }))
    }

    async fn execute_activity_allocation(
        &self,
        request: Request<ExecuteAllocationRequest>,
    ) -> Result<Response<AllocationResponse>, Status> {
        let req = request.into_inner();
        let cmd = ExecuteAllocationCommand {
            controlling_area: req.controlling_area,
            fiscal_year: req.fiscal_year,
            fiscal_period: req.fiscal_period,
            allocation_cycle: req.allocation_cycle,
            allocation_type: "ACTIVITY".to_string(),
            test_run: req.test_run,
        };
        let run_id = self
            .handler
            .execute_allocation(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(AllocationResponse {
            success: true,
            run_id,
            messages: vec![],
        }))
    }

    async fn get_allocation_result(
        &self,
        request: Request<GetAllocationResultRequest>,
    ) -> Result<Response<GetAllocationResultResponse>, Status> {
        let req = request.into_inner();
        let run_id = uuid::Uuid::parse_str(&req.run_id)
            .map_err(|_| Status::invalid_argument("Invalid UUID"))?;
        let run = self
            .repo
            .find_by_id(run_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Run not found"))?;
        Ok(Response::new(GetAllocationResultResponse {
            run_id: run.run_id.to_string(),
            generated_documents: vec![],
            senders: run
                .senders
                .into_iter()
                .map(|s| AllocationSender {
                    sender_object: s.sender_object,
                    sent_amount: Some(common_v1::MonetaryValue {
                        value: s.sent_amount.to_string(),
                        currency_code: s.currency,
                    }),
                    cost_center: s.cost_center.unwrap_or_default(),
                    profit_center: s.profit_center.unwrap_or_default(),
                    segment: s.segment.unwrap_or_default(),
                    internal_order: s.internal_order.unwrap_or_default(),
                    wbs_element: s.wbs_element.unwrap_or_default(),
                })
                .collect(),
            receivers: run
                .receivers
                .into_iter()
                .map(|r| AllocationReceiver {
                    receiver_object: r.receiver_object,
                    received_amount: Some(common_v1::MonetaryValue {
                        value: r.received_amount.to_string(),
                        currency_code: r.currency,
                    }),
                    cost_center: r.cost_center.unwrap_or_default(),
                    profit_center: r.profit_center.unwrap_or_default(),
                    segment: r.segment.unwrap_or_default(),
                    internal_order: r.internal_order.unwrap_or_default(),
                    wbs_element: r.wbs_element.unwrap_or_default(),
                })
                .collect(),
        }))
    }
}
