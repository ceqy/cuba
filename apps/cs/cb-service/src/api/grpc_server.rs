use crate::application::commands::{BillingItem, CreateBillingPlanCommand, RunBillingCommand};
use crate::application::handlers::BillingHandler;
use crate::infrastructure::repository::ContractRepository;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::api::proto::common::v1 as common_v1;
use crate::api::proto::cs::cb::v1 as cb_v1;

use cb_v1::contract_billing_service_server::ContractBillingService;
use cb_v1::*;

pub struct CbServiceImpl {
    handler: Arc<BillingHandler>,
    repo: Arc<ContractRepository>,
}

impl CbServiceImpl {
    pub fn new(handler: Arc<BillingHandler>, repo: Arc<ContractRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl ContractBillingService for CbServiceImpl {
    async fn create_billing_plan(
        &self,
        request: Request<CreateBillingPlanRequest>,
    ) -> Result<Response<BillingPlanResponse>, Status> {
        let req = request.into_inner();
        let cmd = CreateBillingPlanCommand {
            contract_number: req.contract_number.clone(),
            customer_id: "CUST001".to_string(), // Simplified
            validity_start: chrono::Utc::now().date_naive(),
            validity_end: chrono::Utc::now().date_naive() + chrono::Duration::days(365),
            items: req
                .items
                .into_iter()
                .map(|i| BillingItem {
                    planned_date: chrono::Utc::now().date_naive(), // Simplified
                    amount: i
                        .billing_amount
                        .map(|a| a.value.parse().unwrap_or_default())
                        .unwrap_or_default(),
                })
                .collect(),
        };
        self.handler
            .create_billing_plan(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(BillingPlanResponse {
            success: true,
            contract_number: req.contract_number,
            messages: vec![],
        }))
    }

    async fn get_service_contract(
        &self,
        request: Request<GetServiceContractRequest>,
    ) -> Result<Response<ServiceContractDetail>, Status> {
        let req = request.into_inner();
        let contract = self
            .repo
            .find_by_number(&req.contract_number)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Contract not found"))?;
        Ok(Response::new(ServiceContractDetail {
            contract_number: contract.contract_number,
            customer_id: contract.customer_id,
            validity_period: None,
            billing_plan: contract
                .billing_plan
                .into_iter()
                .map(|i| BillingPlanItem {
                    planned_billing_date: None,
                    billing_amount: Some(common_v1::MonetaryValue {
                        value: i.amount.to_string(),
                        currency_code: i.currency,
                    }),
                    status: common_v1::BillingPlanStatus::Planned as i32,
                    invoice_number: i.invoice_number.unwrap_or_default(),
                })
                .collect(),
        }))
    }

    async fn run_billing(
        &self,
        request: Request<RunBillingRequest>,
    ) -> Result<Response<common_v1::JobInfo>, Status> {
        let cmd = RunBillingCommand {
            until_date: chrono::Utc::now().date_naive(),
        };
        let count = self
            .handler
            .run_billing(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(common_v1::JobInfo {
            job_id: uuid::Uuid::new_v4().to_string(),
            job_type: "BILLING_RUN".to_string(),
            status: common_v1::JobStatus::Completed as i32,
            progress_percentage: 100,
            messages: vec![],
            error_detail: format!("{} invoices created", count),
            created_at: None,
            started_at: None,
            completed_at: None,
        }))
    }

    // Stubs
    async fn get_billing_history(
        &self,
        _r: Request<GetBillingHistoryRequest>,
    ) -> Result<Response<BillingHistoryResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn preview_billing_run(
        &self,
        _r: Request<PreviewBillingRunRequest>,
    ) -> Result<Response<PreviewBillingRunResponse>, Status> {
        Err(Status::unimplemented(""))
    }
}
