use tonic::{Request, Response, Status}; use std::sync::Arc; use std::pin::Pin; use tokio_stream::Stream;
use crate::application::handlers::VendorHandler; use crate::infrastructure::repository::VendorRepository;
use crate::api::proto::sc::vs::v1 as vs_v1; use crate::api::proto::common::v1 as common_v1;
use vs_v1::visibility_service_server::VisibilityService; use vs_v1::*;

pub struct VsServiceImpl { _handler: Arc<VendorHandler>, _repo: Arc<VendorRepository> }
impl VsServiceImpl { pub fn new(handler: Arc<VendorHandler>, repo: Arc<VendorRepository>) -> Self { Self { _handler: handler, _repo: repo } } }

#[tonic::async_trait]
impl VisibilityService for VsServiceImpl {
    async fn track_sales_order_flow(&self, request: Request<TrackRequest>) -> Result<Response<SalesOrderFlow>, Status> {
        let req = request.into_inner();
        Ok(Response::new(SalesOrderFlow { sales_order: Some(DocumentStatus { document_number: req.document_number, document_type: common_v1::DocumentType::SalesOrder as i32, status_code: "COMPLETED".to_string(), status_description: "Completed".to_string(), last_updated: Some(prost_types::Timestamp { seconds: chrono::Utc::now().timestamp(), nanos: 0 }), deep_link: "".to_string() }), deliveries: vec![], shipments: vec![], invoices: vec![], payment: None, overall_status: common_v1::FlowStatus::Completed as i32 }))
    }
    async fn track_purchase_order_flow(&self, request: Request<TrackRequest>) -> Result<Response<PurchaseOrderFlow>, Status> {
        let req = request.into_inner();
        Ok(Response::new(PurchaseOrderFlow { purchase_order: Some(DocumentStatus { document_number: req.document_number, document_type: common_v1::DocumentType::PurchaseOrder as i32, status_code: "COMPLETED".to_string(), status_description: "Completed".to_string(), last_updated: Some(prost_types::Timestamp { seconds: chrono::Utc::now().timestamp(), nanos: 0 }), deep_link: "".to_string() }), goods_receipts: vec![], invoices: vec![], payment: None, overall_status: common_v1::FlowStatus::Completed as i32 }))
    }
    type SubscribeToFlowEventsStream = Pin<Box<dyn Stream<Item = Result<FlowEvent, Status>> + Send>>;
    async fn subscribe_to_flow_events(&self, _r: Request<SubscribeRequest>) -> Result<Response<Self::SubscribeToFlowEventsStream>, Status> { Err(Status::unimplemented("Streaming not implemented")) }
    async fn get_flow_summary(&self, _r: Request<GetFlowSummaryRequest>) -> Result<Response<FlowSummary>, Status> {
        Ok(Response::new(FlowSummary { total_active_flows: 10, delayed_flows: 2, completed_flows_today: 25 }))
    }
}
