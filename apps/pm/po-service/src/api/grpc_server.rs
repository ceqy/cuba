use crate::application::commands::{CreatePurchaseOrderCommand, CreatePurchaseOrderItemCommand};
use crate::application::handlers::CreatePurchaseOrderHandler;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::api::proto::common::v1 as common_v1;
use crate::api::proto::pm::po::v1 as po_v1;

use chrono::NaiveDate;
use po_v1::purchase_order_service_server::PurchaseOrderService;
use po_v1::*;

pub struct PoServiceImpl {
    create_handler: Arc<CreatePurchaseOrderHandler>,
}

impl PoServiceImpl {
    pub fn new(create_handler: Arc<CreatePurchaseOrderHandler>) -> Self {
        Self { create_handler }
    }
}

fn to_naive_date(ts: Option<prost_types::Timestamp>) -> Option<NaiveDate> {
    ts.and_then(|t| chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32))
        .map(|dt| dt.date_naive())
}

#[tonic::async_trait]
impl PurchaseOrderService for PoServiceImpl {
    async fn create_purchase_order(
        &self,
        request: Request<CreatePurchaseOrderRequest>,
    ) -> Result<Response<PurchaseOrderResponse>, Status> {
        let req = request.into_inner();
        let header = req
            .header
            .ok_or_else(|| Status::invalid_argument("Header missing"))?;

        // MVP: Enum mapping simplified

        let cmd = CreatePurchaseOrderCommand {
            document_type: header.document_type,
            company_code: header.company_code,
            purchasing_org: header.purchasing_org,
            purchasing_group: header.purchasing_group,
            supplier: header.supplier,
            order_date: to_naive_date(header.order_date),
            currency: header.currency,
            items: req
                .items
                .into_iter()
                .map(|i| {
                    let qty = i.quantity.unwrap_or_default();
                    CreatePurchaseOrderItemCommand {
                        item_number: i.item_number,
                        material: i.material,
                        plant: i.plant,
                        quantity: qty.value.parse().unwrap_or_default(),
                        quantity_unit: qty.unit_code,
                        net_price: i
                            .net_price
                            .unwrap_or_default()
                            .value
                            .parse()
                            .unwrap_or_default(),
                    }
                })
                .collect(),
        };

        let number = self
            .create_handler
            .handle(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(PurchaseOrderResponse {
            success: true,
            order_number: number,
            messages: vec![],
        }))
    }

    // Stubs
    async fn update_purchase_order(
        &self,
        _r: Request<UpdatePurchaseOrderRequest>,
    ) -> Result<Response<PurchaseOrderResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn cancel_purchase_order(
        &self,
        _r: Request<CancelPurchaseOrderRequest>,
    ) -> Result<Response<PurchaseOrderResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn get_purchase_order(
        &self,
        _r: Request<GetPurchaseOrderRequest>,
    ) -> Result<Response<PurchaseOrderDetail>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn list_purchase_orders(
        &self,
        _r: Request<ListPurchaseOrdersRequest>,
    ) -> Result<Response<ListPurchaseOrdersResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn release_purchase_order(
        &self,
        _r: Request<ReleasePurchaseOrderRequest>,
    ) -> Result<Response<ReleasePurchaseOrderResponse>, Status> {
        Err(Status::unimplemented(""))
    }
}
