use crate::application::commands::{CreateBatchCommand, TraceCommand};
use crate::application::handlers::BatchHandler;
use crate::infrastructure::repository::BatchRepository;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::api::proto::common::v1 as common_v1;
use crate::api::proto::sc::bt::v1 as bt_v1;

use bt_v1::batch_traceability_service_server::BatchTraceabilityService;
use bt_v1::*;

pub struct BtServiceImpl {
    handler: Arc<BatchHandler>,
    repo: Arc<BatchRepository>,
}

impl BtServiceImpl {
    pub fn new(handler: Arc<BatchHandler>, repo: Arc<BatchRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl BatchTraceabilityService for BtServiceImpl {
    async fn create_batch(
        &self,
        request: Request<CreateBatchRequest>,
    ) -> Result<Response<BatchResponse>, Status> {
        let req = request.into_inner();
        let cmd = CreateBatchCommand {
            material: req.material,
            plant: req.plant,
            production_date: None,
            expiration_date: None,
            supplier_batch: if req.supplier_batch.is_empty() {
                None
            } else {
                Some(req.supplier_batch)
            },
        };
        let batch_num = self
            .handler
            .create_batch(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(BatchResponse {
            success: true,
            batch_number: batch_num,
            messages: vec![],
        }))
    }

    async fn trace_upstream(
        &self,
        request: Request<TraceRequest>,
    ) -> Result<Response<common_v1::JobInfo>, Status> {
        let req = request.into_inner();
        let cmd = TraceCommand {
            material: req.material,
            batch: req.batch,
            plant: req.plant,
            depth_limit: req.depth_limit,
        };
        let job_id = self
            .handler
            .trace(cmd, "UPSTREAM")
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(common_v1::JobInfo {
            job_id,
            job_type: "TRACE_UPSTREAM".to_string(),
            status: common_v1::JobStatus::Running as i32,
            progress_percentage: 0,
            messages: vec![],
            error_detail: "".to_string(),
            created_at: None,
            started_at: None,
            completed_at: None,
        }))
    }

    async fn trace_downstream(
        &self,
        request: Request<TraceRequest>,
    ) -> Result<Response<common_v1::JobInfo>, Status> {
        let req = request.into_inner();
        let cmd = TraceCommand {
            material: req.material,
            batch: req.batch,
            plant: req.plant,
            depth_limit: req.depth_limit,
        };
        let job_id = self
            .handler
            .trace(cmd, "DOWNSTREAM")
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(common_v1::JobInfo {
            job_id,
            job_type: "TRACE_DOWNSTREAM".to_string(),
            status: common_v1::JobStatus::Running as i32,
            progress_percentage: 0,
            messages: vec![],
            error_detail: "".to_string(),
            created_at: None,
            started_at: None,
            completed_at: None,
        }))
    }

    async fn get_batch_history(
        &self,
        request: Request<GetBatchHistoryRequest>,
    ) -> Result<Response<BatchHistoryResponse>, Status> {
        let req = request.into_inner();
        let batch = self
            .repo
            .find_by_material_batch(&req.material, &req.batch, &req.plant)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Batch not found"))?;
        let events = self
            .repo
            .get_history(batch.batch_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(BatchHistoryResponse {
            events: events
                .into_iter()
                .map(|e| BatchHistoryEvent {
                    event_time: None,
                    event_type: e.event_type,
                    user_id: e.user_id.unwrap_or_default(),
                    details: e.details.unwrap_or_default(),
                    document: e
                        .document_number
                        .map(|d| common_v1::SystemDocumentReference {
                            document_number: d,
                            fiscal_year: 2026,
                            company_code: "".to_string(),
                            document_category: "".to_string(),
                            document_type: e.document_type.unwrap_or_default(),
                        }),
                })
                .collect(),
        }))
    }

    async fn get_trace_result(
        &self,
        _r: Request<GetTraceResultRequest>,
    ) -> Result<Response<TraceabilityTree>, Status> {
        // Stub - would return actual trace tree
        Ok(Response::new(TraceabilityTree {
            root: None,
            generated_at: None,
        }))
    }
}
