use tonic::{Request, Response, Status};
use std::sync::Arc;
use crate::application::commands::{CreateBudgetCommand, PostDirectCostCommand};
use crate::application::handlers::ProjectCostHandler;
use crate::infrastructure::repository::ProjectCostRepository;
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::api::proto::rd::ps::v1 as ps_v1;
use crate::api::proto::common::v1 as common_v1;

use ps_v1::project_cost_controlling_service_server::ProjectCostControllingService;
use ps_v1::*;

pub struct PsServiceImpl {
    handler: Arc<ProjectCostHandler>,
    repo: Arc<ProjectCostRepository>,
}

impl PsServiceImpl {
    pub fn new(handler: Arc<ProjectCostHandler>, repo: Arc<ProjectCostRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl ProjectCostControllingService for PsServiceImpl {

    async fn create_project_budget(
        &self,
        request: Request<CreateBudgetRequest>,
    ) -> Result<Response<BudgetResponse>, Status> {
        let req = request.into_inner();
        let amount = req.amount.map(|a| Decimal::from_str(&a.value).unwrap_or_default()).unwrap_or_default();
        let cmd = CreateBudgetCommand {
            wbs_element: req.project_or_wbs_element,
            fiscal_year: req.fiscal_year.parse().unwrap_or(2026),
            amount,
        };
        let doc_id = self.handler.create_budget(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(BudgetResponse {
            success: true,
            document_number: doc_id,
            messages: vec![],
        }))
    }

    async fn post_direct_cost(
        &self,
        request: Request<PostDirectCostRequest>,
    ) -> Result<Response<common_v1::JobInfo>, Status> {
        let req = request.into_inner();
        let amount = req.amount.map(|a| Decimal::from_str(&a.value).unwrap_or_default()).unwrap_or_default();
        let cmd = PostDirectCostCommand {
            wbs_element: req.wbs_element,
            cost_element: req.cost_element,
            amount,
            posting_date: chrono::Utc::now().date_naive(),
            description: if req.text.is_empty() { None } else { Some(req.text) },
        };
        let doc_num = self.handler.post_direct_cost(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(common_v1::JobInfo {
            job_id: doc_num,
            job_type: "COST_POSTING".to_string(),
            status: common_v1::JobStatus::Completed as i32,
            progress_percentage: 100,
            messages: vec![],
            error_detail: "".to_string(),
            created_at: None,
            started_at: None,
            completed_at: None,
        }))
    }

    async fn get_project_cost_report(
        &self,
        request: Request<GetProjectCostReportRequest>,
    ) -> Result<Response<ProjectCostReport>, Status> {
        let req = request.into_inner();
        let (budget, actual) = self.repo.get_cost_report(&req.project_or_wbs_element).await
            .map_err(|e| Status::internal(e.to_string()))?;
        let available = budget - actual;
        Ok(Response::new(ProjectCostReport {
            project_or_wbs_element: req.project_or_wbs_element,
            description: "".to_string(),
            planned_cost: None,
            actual_cost: Some(common_v1::MonetaryValue { value: actual.to_string(), currency_code: "CNY".to_string() }),
            commitment_cost: None,
            budget: Some(common_v1::MonetaryValue { value: budget.to_string(), currency_code: "CNY".to_string() }),
            available_amount: Some(common_v1::MonetaryValue { value: available.to_string(), currency_code: "CNY".to_string() }),
            cost_details: vec![],
            budget_status: common_v1::BudgetStatus::Approved as i32,
        }))
    }

    // Stubs
    async fn list_project_costs(&self, _r: Request<ListProjectCostsRequest>) -> Result<Response<ListProjectCostsResponse>, Status> { Err(Status::unimplemented("")) }
    async fn update_cost(&self, _r: Request<UpdateCostRequest>) -> Result<Response<PostDirectCostResponse>, Status> { Err(Status::unimplemented("")) }
}
