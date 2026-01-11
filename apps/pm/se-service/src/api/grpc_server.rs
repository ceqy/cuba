use tonic::{Request, Response, Status};
use std::sync::Arc;
use crate::application::commands::{CreateRFQCommand, SubmitQuoteCommand, AwardCommand};
use crate::application::handlers::SourcingHandler;
use crate::infrastructure::repository::SourcingRepository;

use crate::api::proto::pm::se::v1 as se_v1;
use crate::api::proto::common::v1 as common_v1;

use se_v1::sourcing_event_service_server::SourcingEventService;
use se_v1::*;

pub struct SeServiceImpl {
    handler: Arc<SourcingHandler>,
    repo: Arc<SourcingRepository>,
}

impl SeServiceImpl {
    pub fn new(handler: Arc<SourcingHandler>, repo: Arc<SourcingRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl SourcingEventService for SeServiceImpl {

    async fn create_rfq(
        &self,
        request: Request<CreateRfqRequest>,
    ) -> Result<Response<RfqResponse>, Status> {
        let req = request.into_inner();
        let rfq_detail = req.rfq.ok_or_else(|| Status::invalid_argument("RFQ detail required"))?;
        let cmd = CreateRFQCommand {
            company_code: rfq_detail.company_code,
            purchasing_org: if rfq_detail.purchasing_org.is_empty() { None } else { Some(rfq_detail.purchasing_org) },
        };
        let rfq_num = self.handler.create_rfq(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(RfqResponse {
            success: true,
            rfq_number: rfq_num,
            messages: vec![],
        }))
    }

    async fn get_rfq(
        &self,
        request: Request<GetRfqRequest>,
    ) -> Result<Response<RfqDetail>, Status> {
        let req = request.into_inner();
        let rfq = self.repo.find_rfq_by_number(&req.rfq_number).await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("RFQ not found"))?;
        Ok(Response::new(RfqDetail {
            rfq_number: rfq.rfq_number,
            company_code: rfq.company_code,
            purchasing_org: rfq.purchasing_org.unwrap_or_default(),
            quote_deadline: rfq.quote_deadline.map(|d| prost_types::Timestamp { seconds: d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(), nanos: 0 }),
            items: rfq.items.into_iter().map(|item| RfqItem {
                item_number: item.item_number,
                material: item.material,
                description: item.description.unwrap_or_default(),
                quantity: item.quantity.map(|q| common_v1::QuantityValue { value: q.to_string(), unit_code: item.unit.clone() }),
                delivery_date: item.delivery_date.map(|d| prost_types::Timestamp { seconds: d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(), nanos: 0 }),
                item_category: common_v1::ItemCategory::Standard as i32,
            }).collect(),
            invited_suppliers: vec![],
            document_type: common_v1::DocumentType::Rfq as i32,
            status: common_v1::ReleaseStatus::NotReleased as i32,
        }))
    }

    async fn submit_supplier_quote(
        &self,
        request: Request<SubmitQuoteRequest>,
    ) -> Result<Response<QuoteResponse>, Status> {
        let req = request.into_inner();
        let quote_detail = req.quote.ok_or_else(|| Status::invalid_argument("Quote detail required"))?;
        let cmd = SubmitQuoteCommand {
            rfq_number: quote_detail.rfq_number,
            supplier_id: quote_detail.supplier_id,
        };
        let quote_num = self.handler.submit_quote(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(QuoteResponse {
            success: true,
            quote_number: quote_num,
            messages: vec![],
        }))
    }

    async fn award_quote(
        &self,
        request: Request<AwardQuoteRequest>,
    ) -> Result<Response<QuoteResponse>, Status> {
        let req = request.into_inner();
        let cmd = AwardCommand { quote_number: req.quote_number.clone() };
        let _success = self.handler.award_quote(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(QuoteResponse {
            success: true,
            quote_number: req.quote_number,
            messages: vec![],
        }))
    }

    async fn publish_rfq(&self, _r: Request<PublishRfqRequest>) -> Result<Response<RfqResponse>, Status> { Err(Status::unimplemented("")) }
    async fn close_rfq(&self, _r: Request<CloseRfqRequest>) -> Result<Response<RfqResponse>, Status> { Err(Status::unimplemented("")) }
    async fn get_supplier_quote(&self, _r: Request<GetQuoteRequest>) -> Result<Response<QuoteDetail>, Status> { Err(Status::unimplemented("")) }
}
