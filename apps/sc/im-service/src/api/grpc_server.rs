use crate::application::commands::{
    GetStockOverviewQuery, PostStockMovementCommand, StockMovementItemCommand,
};
use crate::application::handlers::{GetStockOverviewHandler, PostStockMovementHandler};
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::api::proto::common::v1 as common_v1;
use crate::api::proto::sc::im::v1 as im_v1;

use chrono::NaiveDate;
use im_v1::inventory_management_service_server::InventoryManagementService;
use im_v1::*;
use rust_decimal::Decimal;

pub struct ImServiceImpl {
    post_handler: Arc<PostStockMovementHandler>,
    get_stock_handler: Arc<GetStockOverviewHandler>,
}

impl ImServiceImpl {
    pub fn new(
        post_handler: Arc<PostStockMovementHandler>,
        get_stock_handler: Arc<GetStockOverviewHandler>,
    ) -> Self {
        Self {
            post_handler,
            get_stock_handler,
        }
    }
}

// Helpers
fn to_proto_qty(qty: Decimal, unit: &str) -> common_v1::QuantityValue {
    common_v1::QuantityValue {
        value: qty.to_string(),
        unit_code: unit.to_string(),
    }
}

fn to_naive_date(ts: Option<prost_types::Timestamp>) -> Option<NaiveDate> {
    ts.and_then(|t| chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32))
        .map(|dt| dt.date_naive())
}

#[tonic::async_trait]
impl InventoryManagementService for ImServiceImpl {
    async fn post_stock_movement(
        &self,
        request: Request<PostStockMovementRequest>,
    ) -> Result<Response<common_v1::StockMovementResponse>, Status> {
        let req = request.into_inner();
        let header = req
            .header
            .ok_or_else(|| Status::invalid_argument("Header missing"))?;

        let parse_date = |s: &str| -> NaiveDate {
            NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .unwrap_or_else(|_| chrono::Utc::now().date_naive())
        };

        let cmd = PostStockMovementCommand {
            posting_date: parse_date(&header.posting_date),
            document_date: parse_date(&header.document_date),
            header_text: Some(header.header_text),
            reference_document: None, // Not present in common.v1 header
            items: req
                .items
                .into_iter()
                .map(|i| {
                    let q = i.quantity.unwrap_or_default();
                    StockMovementItemCommand {
                        movement_type: i.move_type.to_string(),
                        material: i.material,
                        plant: i.plant,
                        storage_location: i.storage_location,
                        batch: if i.batch.is_empty() {
                            None
                        } else {
                            Some(i.batch)
                        },
                        quantity: q.value.parse().unwrap_or_default(),
                        unit_of_measure: q.unit_code,
                    }
                })
                .collect(),
        };

        let number = self
            .post_handler
            .handle(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(common_v1::StockMovementResponse {
            success: true,
            material_document_number: number,
            material_document_year: 2024, // simplified
            messages: vec![],
        }))
    }

    async fn get_stock_overview(
        &self,
        request: Request<GetStockOverviewRequest>,
    ) -> Result<Response<GetStockOverviewResponse>, Status> {
        let req = request.into_inner();
        let stocks = self
            .get_stock_handler
            .handle(GetStockOverviewQuery {
                material: req.material,
                plant: req.plant,
                storage_location: if req.storage_locations.is_empty() {
                    None
                } else {
                    Some(req.storage_locations[0].clone())
                },
            })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetStockOverviewResponse {
            items: stocks
                .into_iter()
                .map(|s| StockOverviewItem {
                    material: s.material,
                    plant: s.plant,
                    storage_location: s.storage_location,
                    batch: s.batch,
                    unrestricted_stock: Some(to_proto_qty(s.unrestricted_quantity, &s.base_unit)),
                    quality_inspection_stock: Some(to_proto_qty(
                        s.quality_inspection_quantity,
                        &s.base_unit,
                    )),
                    blocked_stock: Some(to_proto_qty(s.blocked_quantity, &s.base_unit)),
                })
                .collect(),
            pagination: None,
        }))
    }

    // Stubs
    async fn stream_stock_list(
        &self,
        _r: Request<ListStockRequest>,
    ) -> Result<Response<Self::StreamStockListStream>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn check_stock_availability(
        &self,
        _r: Request<CheckStockAvailabilityRequest>,
    ) -> Result<Response<StockAvailabilityResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn reverse_material_document(
        &self,
        _r: Request<ReverseMaterialDocumentRequest>,
    ) -> Result<Response<common_v1::StockMovementResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn create_reservation(
        &self,
        _r: Request<CreateReservationRequest>,
    ) -> Result<Response<ReservationResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn cancel_reservation(
        &self,
        _r: Request<CancelReservationRequest>,
    ) -> Result<Response<ReservationResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn create_physical_inventory_document(
        &self,
        _r: Request<CreatePiDocumentRequest>,
    ) -> Result<Response<PiDocumentResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn enter_inventory_count(
        &self,
        _r: Request<EnterInventoryCountRequest>,
    ) -> Result<Response<EnterInventoryCountResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn post_inventory_difference(
        &self,
        _r: Request<PostInventoryDifferenceRequest>,
    ) -> Result<Response<common_v1::StockMovementResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    type StreamStockListStream = tokio_stream::wrappers::ReceiverStream<Result<StockItem, Status>>;
}
