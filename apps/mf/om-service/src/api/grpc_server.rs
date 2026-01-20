use crate::application::commands::{
    ComponentInput, CreateOrderCommand, OrderItemInput, PostComponentsCommand, ReceiveGoodsCommand,
};
use crate::application::handlers::SubcontractingHandler;
use crate::infrastructure::repository::SubcontractingRepository;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::api::proto::common::v1 as common_v1;
use crate::api::proto::mf::om::v1 as om_v1;

use om_v1::outsourced_manufacturing_service_server::OutsourcedManufacturingService;
use om_v1::*;

pub struct OmServiceImpl {
    handler: Arc<SubcontractingHandler>,
    repo: Arc<SubcontractingRepository>,
}

impl OmServiceImpl {
    pub fn new(handler: Arc<SubcontractingHandler>, repo: Arc<SubcontractingRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl OutsourcedManufacturingService for OmServiceImpl {
    async fn create_subcontracting_order(
        &self,
        request: Request<CreateSubcontractingOrderRequest>,
    ) -> Result<Response<SubcontractingOrderResponse>, Status> {
        let req = request.into_inner();
        let cmd = CreateOrderCommand {
            supplier: req.supplier,
            company_code: req.company_code,
            purchasing_org: if req.purchasing_org.is_empty() {
                None
            } else {
                Some(req.purchasing_org)
            },
            items: req
                .items
                .into_iter()
                .map(|i| OrderItemInput {
                    material: i.material,
                    quantity: Decimal::from_str(&i.quantity.map(|q| q.value).unwrap_or_default())
                        .unwrap_or_default(),
                    plant: if i.plant.is_empty() {
                        None
                    } else {
                        Some(i.plant)
                    },
                    components: i
                        .components
                        .into_iter()
                        .map(|c| ComponentInput {
                            material: c.component_material,
                            quantity: Decimal::from_str(
                                &c.required_quantity.map(|q| q.value).unwrap_or_default(),
                            )
                            .unwrap_or_default(),
                        })
                        .collect(),
                })
                .collect(),
        };
        let po_num = self
            .handler
            .create_order(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(SubcontractingOrderResponse {
            success: true,
            purchase_order_number: po_num,
            messages: vec![],
        }))
    }

    async fn get_subcontracting_order(
        &self,
        request: Request<GetSubcontractingOrderRequest>,
    ) -> Result<Response<SubcontractingOrderDetail>, Status> {
        let req = request.into_inner();
        let order = self
            .repo
            .find_by_po_number(&req.purchase_order_number)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Order not found"))?;
        Ok(Response::new(SubcontractingOrderDetail {
            purchase_order_number: order.purchase_order_number,
            supplier: order.supplier,
            finished_goods_items: order
                .items
                .into_iter()
                .map(|item| SubcontractingItem {
                    item_number: item.item_number,
                    finished_good_material: item.finished_good_material,
                    order_quantity: item.order_quantity.map(|q| common_v1::QuantityValue {
                        value: q.to_string(),
                        unit_code: item.unit.clone(),
                    }),
                    received_quantity: Some(common_v1::QuantityValue {
                        value: item.received_quantity.to_string(),
                        unit_code: item.unit.clone(),
                    }),
                    unit: item.unit,
                    components: item
                        .components
                        .into_iter()
                        .map(|c| SubcontractingComponent {
                            component_material: c.component_material,
                            required_quantity: c.required_quantity.map(|q| {
                                common_v1::QuantityValue {
                                    value: q.to_string(),
                                    unit_code: c.unit.clone(),
                                }
                            }),
                            issued_quantity: Some(common_v1::QuantityValue {
                                value: c.issued_quantity.to_string(),
                                unit_code: c.unit.clone(),
                            }),
                            unit: c.unit,
                        })
                        .collect(),
                })
                .collect(),
        }))
    }

    async fn post_components_to_supplier(
        &self,
        request: Request<PostComponentsRequest>,
    ) -> Result<Response<PostComponentsResponse>, Status> {
        let req = request.into_inner();
        let cmd = PostComponentsCommand {
            po_number: req.purchase_order_number,
        };
        let doc_num = self
            .handler
            .post_components(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(PostComponentsResponse {
            material_document_number: doc_num,
            messages: vec![],
        }))
    }

    async fn receive_finished_goods_from_supplier(
        &self,
        request: Request<ReceiveFinishedGoodsRequest>,
    ) -> Result<Response<ReceiveFinishedGoodsResponse>, Status> {
        let req = request.into_inner();
        let cmd = ReceiveGoodsCommand {
            po_number: req.purchase_order_number,
        };
        let doc_num = self
            .handler
            .receive_goods(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ReceiveFinishedGoodsResponse {
            material_document_number: doc_num,
            messages: vec![],
        }))
    }
}
