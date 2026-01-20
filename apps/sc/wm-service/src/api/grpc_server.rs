use crate::application::commands::{ConfirmTOCommand, CreateTOCommand};
use crate::application::handlers::WarehouseHandler;
use crate::infrastructure::repository::TransferOrderRepository;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::api::proto::common::v1 as common_v1;
use crate::api::proto::sc::wm::v1 as wm_v1;

use wm_v1::warehouse_operations_service_server::WarehouseOperationsService;
use wm_v1::*;

pub struct WmServiceImpl {
    handler: Arc<WarehouseHandler>,
    repo: Arc<TransferOrderRepository>,
}

impl WmServiceImpl {
    pub fn new(handler: Arc<WarehouseHandler>, repo: Arc<TransferOrderRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl WarehouseOperationsService for WmServiceImpl {
    async fn create_transfer_order(
        &self,
        request: Request<CreateTransferOrderRequest>,
    ) -> Result<Response<TransferOrderResponse>, Status> {
        let req = request.into_inner();
        let mvt = match req.movement_type {
            1 => "101",
            2 => "102",
            3 => "301",
            4 => "311",
            _ => "999",
        };
        let cmd = CreateTOCommand {
            warehouse_number: req.warehouse_number,
            movement_type: mvt.to_string(),
            reference_doc_number: if req.reference_document_number.is_empty() {
                None
            } else {
                Some(req.reference_document_number)
            },
        };
        let to_num = self
            .handler
            .create_transfer_order(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(TransferOrderResponse {
            success: true,
            transfer_order_number: to_num,
            messages: vec![],
        }))
    }

    async fn confirm_transfer_order(
        &self,
        request: Request<ConfirmTransferOrderRequest>,
    ) -> Result<Response<ConfirmTransferOrderResponse>, Status> {
        let req = request.into_inner();
        let cmd = ConfirmTOCommand {
            warehouse_number: req.warehouse_number,
            to_number: req.transfer_order_number,
        };
        self.handler
            .confirm_transfer_order(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ConfirmTransferOrderResponse {
            success: true,
            messages: vec![],
        }))
    }

    async fn get_transfer_order(
        &self,
        request: Request<GetTransferOrderRequest>,
    ) -> Result<Response<TransferOrderDetail>, Status> {
        let req = request.into_inner();
        let to = self
            .repo
            .find_by_number(&req.warehouse_number, &req.transfer_order_number)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("TO not found"))?;
        Ok(Response::new(TransferOrderDetail {
            transfer_order_number: to.to_number,
            header: Some(TransferOrderHeader {
                warehouse_number: to.warehouse_number,
                movement_type: common_v1::MovementType::GoodsReceipt as i32,
                reference_document_type: to.reference_doc_type.unwrap_or_default(),
                reference_document_number: to.reference_doc_number.unwrap_or_default(),
                created_by: to.created_by.unwrap_or_default(),
            }),
            items: to
                .items
                .into_iter()
                .map(|i| TransferOrderItem {
                    item_number: i.item_number,
                    material: i.material,
                    target_quantity: Some(common_v1::QuantityValue {
                        value: i.target_quantity.to_string(),
                        unit_code: i.unit.clone(),
                    }),
                    actual_quantity: Some(common_v1::QuantityValue {
                        value: i.actual_quantity.to_string(),
                        unit_code: i.unit,
                    }),
                    source_storage_type: i.src_storage_type.unwrap_or_default(),
                    source_storage_bin: i.src_storage_bin.unwrap_or_default(),
                    destination_storage_type: i.dst_storage_type.unwrap_or_default(),
                    destination_storage_bin: i.dst_storage_bin.unwrap_or_default(),
                    batch: i.batch.unwrap_or_default(),
                    confirmation_indicator: i.confirmed,
                })
                .collect(),
            audit_data: None,
        }))
    }

    // Stubs
    async fn partial_confirm_item(
        &self,
        _r: Request<PartialConfirmItemRequest>,
    ) -> Result<Response<ConfirmTransferOrderResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn print_transfer_order_label(
        &self,
        _r: Request<PrintLabelRequest>,
    ) -> Result<Response<PrintLabelResponse>, Status> {
        Err(Status::unimplemented(""))
    }
}
