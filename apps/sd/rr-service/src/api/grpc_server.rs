use crate::application::commands::{CreateContractCommand, RunPostingCommand};
use crate::application::handlers::RevenueHandler;
use crate::infrastructure::repository::RevenueRepository;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::api::proto::common::v1 as common_v1;
use crate::api::proto::sd::rr::v1 as rr_v1;

use rr_v1::revenue_recognition_service_server::RevenueRecognitionService;
use rr_v1::*;

pub struct RrServiceImpl {
    handler: Arc<RevenueHandler>,
    repo: Arc<RevenueRepository>,
}

impl RrServiceImpl {
    pub fn new(handler: Arc<RevenueHandler>, repo: Arc<RevenueRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl RevenueRecognitionService for RrServiceImpl {
    async fn create_revenue_contract(
        &self,
        request: Request<CreateRevenueContractRequest>,
    ) -> Result<Response<RevenueContractResponse>, Status> {
        let req = request.into_inner();
        let cmd = CreateContractCommand {
            source_document_number: req.source_document_number,
            source_document_type: format!("{}", req.source_document_type),
            company_code: req.company_code,
            customer: req.customer,
        };
        let contract_num = self
            .handler
            .create_contract(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(RevenueContractResponse {
            success: true,
            revenue_contract_number: contract_num,
            messages: vec![],
        }))
    }

    async fn get_performance_obligations(
        &self,
        request: Request<GetPerformanceObligationsRequest>,
    ) -> Result<Response<GetPerformanceObligationsResponse>, Status> {
        let req = request.into_inner();
        let c = self
            .repo
            .find_contract_by_number(&req.revenue_contract_number)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Contract not found"))?;
        Ok(Response::new(GetPerformanceObligationsResponse {
            obligations: c
                .obligations
                .into_iter()
                .map(|pob| PerformanceObligation {
                    pob_id: pob.pob_code,
                    revenue_contract_number: c.contract_number.clone(),
                    source_document: c.source_document_number.clone(),
                    description: pob.description.unwrap_or_default(),
                    allocated_price: pob.allocated_price.map(|a| common_v1::MonetaryValue {
                        value: a.to_string(),
                        currency_code: pob.currency.clone(),
                    }),
                    recognition_method: pob.recognition_method,
                    recognized_revenue: Some(common_v1::MonetaryValue {
                        value: pob.recognized_revenue.to_string(),
                        currency_code: pob.currency.clone(),
                    }),
                    deferred_revenue: Some(common_v1::MonetaryValue {
                        value: pob.deferred_revenue.to_string(),
                        currency_code: pob.currency,
                    }),
                    item_category: common_v1::ItemCategory::Standard as i32,
                })
                .collect(),
        }))
    }

    async fn run_revenue_posting(
        &self,
        request: Request<RunRevenuePostingRequest>,
    ) -> Result<Response<RunRevenuePostingResponse>, Status> {
        let req = request.into_inner();
        let cmd = RunPostingCommand {
            company_code: req.company_code,
            posting_period: req.posting_period,
        };
        let (run_id, doc_count) = self
            .handler
            .run_posting(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(RunRevenuePostingResponse {
            run_id,
            success: true,
            documents_created: doc_count,
            messages: vec![],
        }))
    }

    async fn reverse_revenue_posting(
        &self,
        _r: Request<ReverseRevenuePostingRequest>,
    ) -> Result<Response<RevenuePostingResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn get_revenue_postings(
        &self,
        _r: Request<GetRevenuePostingsRequest>,
    ) -> Result<Response<GetRevenuePostingsResponse>, Status> {
        Err(Status::unimplemented(""))
    }
}
