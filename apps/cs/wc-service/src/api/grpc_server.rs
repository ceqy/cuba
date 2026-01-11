use tonic::{Request, Response, Status};
use std::sync::Arc;
use crate::application::commands::{CreateClaimCommand, AdjudicateClaimCommand};
use crate::application::handlers::ClaimHandler;
use crate::infrastructure::repository::ClaimRepository;
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::api::proto::cs::wc::v1 as wc_v1;
use crate::api::proto::common::v1 as common_v1;

use wc_v1::warranty_claims_service_server::WarrantyClaimsService;
use wc_v1::*;

pub struct WcServiceImpl {
    handler: Arc<ClaimHandler>,
    repo: Arc<ClaimRepository>,
}

impl WcServiceImpl {
    pub fn new(handler: Arc<ClaimHandler>, repo: Arc<ClaimRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl WarrantyClaimsService for WcServiceImpl {

    async fn create_claim(
        &self,
        request: Request<CreateClaimRequest>,
    ) -> Result<Response<ClaimResponse>, Status> {
        let req = request.into_inner();
        let claim = req.claim.unwrap_or_default();
        let amount = claim.claimed_amount.map(|a| Decimal::from_str(&a.value).unwrap_or_default()).unwrap_or_default();
        let cmd = CreateClaimCommand {
            customer_id: claim.customer_id,
            product_id: claim.product_or_equipment_id,
            failure_date: chrono::Utc::now().date_naive(),
            failure_description: if claim.failure_description.is_empty() { None } else { Some(claim.failure_description) },
            claimed_amount: amount,
        };
        let id = self.handler.create_claim(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ClaimResponse {
            success: true,
            claim_id: id,
            messages: vec![],
        }))
    }

    async fn get_claim(
        &self,
        request: Request<GetClaimRequest>,
    ) -> Result<Response<ClaimDetail>, Status> {
        let req = request.into_inner();
        let id = uuid::Uuid::parse_str(&req.claim_id)
            .map_err(|_| Status::invalid_argument("Invalid UUID"))?;
        let c = self.repo.find_by_id(id).await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Claim not found"))?;
        Ok(Response::new(ClaimDetail {
            claim_id: c.claim_id.to_string(),
            customer_id: c.customer_id,
            product_or_equipment_id: c.product_id,
            failure_date: None,
            failure_description: c.failure_description.unwrap_or_default(),
            claimed_amount: Some(common_v1::MonetaryValue { value: c.claimed_amount.to_string(), currency_code: c.currency }),
            status: common_v1::ClaimStatus::UnderReview as i32,
            adjudication: c.adjudication.map(|a| AdjudicationResult {
                adjudicated_by: a.adjudicated_by,
                adjudication_date: None,
                approved_amount: a.approved_amount.map(|v| common_v1::MonetaryValue { value: v.to_string(), currency_code: a.currency }),
                notes: a.notes.unwrap_or_default(),
            }),
            attachment_urls: c.attachment_urls,
        }))
    }

    async fn adjudicate_claim(
        &self,
        request: Request<AdjudicateClaimRequest>,
    ) -> Result<Response<common_v1::JobInfo>, Status> {
        let req = request.into_inner();
        let id = uuid::Uuid::parse_str(&req.claim_id)
            .map_err(|_| Status::invalid_argument("Invalid UUID"))?;
        let new_status = match req.new_status {
            2 => "APPROVED",
            3 => "REJECTED",
            _ => "IN_REVIEW"
        };
        let result = req.result.unwrap_or_default();
        let cmd = AdjudicateClaimCommand {
            claim_id: id,
            new_status: new_status.to_string(),
            approved_amount: result.approved_amount.map(|a| Decimal::from_str(&a.value).unwrap_or_default()),
            notes: if result.notes.is_empty() { None } else { Some(result.notes) },
        };
        let adj_id = self.handler.adjudicate(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(common_v1::JobInfo {
            job_id: adj_id,
            job_type: "ADJUDICATION".to_string(),
            status: common_v1::JobStatus::Completed as i32,
            progress_percentage: 100,
            messages: vec![],
            error_detail: "".to_string(),
            created_at: None,
            started_at: None,
            completed_at: None,
        }))
    }

    // Stubs
    async fn update_claim(&self, _r: Request<UpdateClaimRequest>) -> Result<Response<ClaimResponse>, Status> { Err(Status::unimplemented("")) }
    async fn list_claims(&self, _r: Request<ListClaimsRequest>) -> Result<Response<ListClaimsResponse>, Status> { Err(Status::unimplemented("")) }
    async fn add_evidence_attachment(&self, _r: Request<AddEvidenceAttachmentRequest>) -> Result<Response<ClaimResponse>, Status> { Err(Status::unimplemented("")) }
    async fn close_claim(&self, _r: Request<CloseClaimRequest>) -> Result<Response<ClaimResponse>, Status> { Err(Status::unimplemented("")) }
}
