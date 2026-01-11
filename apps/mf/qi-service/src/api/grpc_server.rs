use tonic::{Request, Response, Status};
use std::sync::Arc;
use crate::application::commands::{CreateInspectionLotCommand, RecordResultCommand, MakeUsageDecisionCommand};
use crate::application::handlers::InspectionHandler;
use crate::infrastructure::repository::InspectionLotRepository;

use crate::api::proto::mf::qi::v1 as qi_v1;
use crate::api::proto::common::v1 as common_v1;

use qi_v1::quality_inspection_service_server::QualityInspectionService;
use qi_v1::*;

pub struct QiServiceImpl {
    handler: Arc<InspectionHandler>,
    repo: Arc<InspectionLotRepository>,
}

impl QiServiceImpl {
    pub fn new(handler: Arc<InspectionHandler>, repo: Arc<InspectionLotRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl QualityInspectionService for QiServiceImpl {

    async fn create_inspection_lot(
        &self,
        request: Request<CreateInspectionLotRequest>,
    ) -> Result<Response<InspectionLotResponse>, Status> {
        let req = request.into_inner();
        let cmd = CreateInspectionLotCommand {
            material: req.material,
            plant: req.plant,
            quantity: req.quantity.unwrap_or_default().value.parse().unwrap_or_default(),
            origin: req.origin,
        };
        
        let lot_num = self.handler.create_lot(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
            
        Ok(Response::new(InspectionLotResponse {
            success: true,
            inspection_lot_number: lot_num,
            messages: vec![],
        }))
    }

    async fn get_inspection_lot(
        &self,
        request: Request<GetInspectionLotRequest>,
    ) -> Result<Response<InspectionLotDetail>, Status> {
        let req = request.into_inner();
        let lot = self.repo.find_by_number(&req.inspection_lot_number).await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Lot not found"))?;

        Ok(Response::new(InspectionLotDetail {
            inspection_lot_number: lot.inspection_lot_number,
            material: lot.material,
            plant: lot.plant,
            lot_origin: common_v1::InspectionLotOrigin::GoodsReceipt as i32, // simplified
            lot_quantity: Some(common_v1::QuantityValue {
                value: lot.lot_quantity.to_string(),
                unit_code: lot.quantity_unit,
            }),
            creation_date: None, // Simplified
            characteristics: lot.characteristics.into_iter().map(|c| InspectionCharacteristic {
                characteristic_number: c.characteristic_number,
                description: c.description.unwrap_or_default(),
                inspection_method: c.inspection_method.unwrap_or_default(),
                result_value: c.result_value.unwrap_or_default(),
                result_status: c.result_status,
                unit: "".to_string(),
            }).collect(),
            usage_decision: common_v1::UsageDecision::Unspecified as i32, // Simplified
        }))
    }

    async fn record_inspection_results(
        &self,
        request: Request<RecordResultsRequest>,
    ) -> Result<Response<RecordResultsResponse>, Status> {
        let req = request.into_inner();
        
        for res in req.results {
            let cmd = RecordResultCommand {
                lot_number: req.inspection_lot_number.clone(),
                characteristic_number: res.characteristic_number,
                value: res.value,
            };
            self.handler.record_result(cmd).await
                .map_err(|e| Status::internal(e.to_string()))?;
        }

        Ok(Response::new(RecordResultsResponse {
            success: true,
            messages: vec![],
        }))
    }

    async fn make_usage_decision(
        &self,
        request: Request<MakeUsageDecisionRequest>,
    ) -> Result<Response<MakeUsageDecisionResponse>, Status> {
        let req = request.into_inner();
        
        // Map enum int to string? Or just use "A" "R"?
        // Protocol defines UsageDecision enum. We just need to persist it.
        // For MVP, we pass enum int as string or just "A"/"R" based on value.
        // Let's assume passed code is meaningful.
        let ud_code = match req.decision_code {
            1 => "A", // Accept
            2 => "R", // Reject
            _ => "?",
        };

        let cmd = MakeUsageDecisionCommand {
            lot_number: req.inspection_lot_number,
            ud_code: ud_code.to_string(),
            note: req.note,
        };

        self.handler.make_usage_decision(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(MakeUsageDecisionResponse {
            success: true,
            messages: vec![],
        }))
    }

    // Stubs
    async fn list_inspection_lots(&self, _r: Request<ListInspectionLotsRequest>) -> Result<Response<ListInspectionLotsResponse>, Status> { Err(Status::unimplemented("")) }

}
