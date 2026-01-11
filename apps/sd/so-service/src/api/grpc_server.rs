use tonic::{Request, Response, Status};
use std::sync::Arc;
use crate::application::commands::{CreateSalesOrderCommand, CreateSalesOrderItemCommand, GetSalesOrderQuery, ListSalesOrdersQuery};
use crate::application::handlers::{CreateSalesOrderHandler, GetSalesOrderHandler, ListSalesOrdersHandler};

use crate::api::proto::sd::so::v1 as so_v1;
use crate::api::proto::common::v1 as common_v1;

use so_v1::sales_order_fulfillment_service_server::SalesOrderFulfillmentService;
use so_v1::*;
use rust_decimal::Decimal;
use chrono::NaiveDate;

pub struct SoServiceImpl {
    create_handler: Arc<CreateSalesOrderHandler>,
    get_handler: Arc<GetSalesOrderHandler>,
    list_handler: Arc<ListSalesOrdersHandler>,
}

impl SoServiceImpl {
    pub fn new(
        create_handler: Arc<CreateSalesOrderHandler>,
        get_handler: Arc<GetSalesOrderHandler>,
        list_handler: Arc<ListSalesOrdersHandler>,
    ) -> Self {
        Self {
            create_handler,
            get_handler,
            list_handler,
        }
    }
}

// Helpers
fn to_proto_money(amount: Decimal, currency: &str) -> common_v1::MonetaryValue {
    common_v1::MonetaryValue {
        value: amount.to_string(),
        currency_code: currency.to_string(),
    }
}

fn to_timestamp(dt: chrono::DateTime<chrono::Utc>) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

fn to_naive_date(ts: Option<prost_types::Timestamp>) -> Option<NaiveDate> {
    ts.and_then(|t| chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32)).map(|dt| dt.date_naive())
}

#[tonic::async_trait]
impl SalesOrderFulfillmentService for SoServiceImpl {

    async fn create_sales_order(
        &self,
        request: Request<CreateSalesOrderRequest>,
    ) -> Result<Response<SalesOrderResponse>, Status> {
        let req = request.into_inner();
        let header = req.header.ok_or_else(|| Status::invalid_argument("Header missing"))?;
        
        let cmd = CreateSalesOrderCommand {
            order_type: "OR".to_string(), // Default for now
            sales_org: header.sales_org,
            distribution_channel: header.distribution_channel,
            division: header.division,
            sold_to_party: header.sold_to_party,
            ship_to_party: Some(header.ship_to_party),
            customer_po: Some(header.customer_po),
            customer_po_date: to_naive_date(header.customer_po_date),
            requested_delivery_date: to_naive_date(header.requested_delivery_date),
            currency: header.currency,
            items: req.items.into_iter().map(|i| CreateSalesOrderItemCommand {
                 item_number: i.item_number,
                 material: i.material,
                 order_quantity: i.order_quantity.parse::<Decimal>().unwrap_or_default(),
                 sales_unit: i.sales_unit,
                 plant: Some(i.plant),
                 storage_location: Some(i.storage_location),
            }).collect(),
        };

        let order = self.create_handler.handle(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(SalesOrderResponse {
            success: true,
            order_number: order.order_number,
            messages: vec![],
        }))
    }

    async fn get_sales_order(
        &self,
        request: Request<GetSalesOrderRequest>,
    ) -> Result<Response<SalesOrderDetail>, Status> {
        let req = request.into_inner();
        let order = self.get_handler.handle(GetSalesOrderQuery { order_number: req.order_number }).await
            .map_err(|e| Status::internal(e.to_string()))?;
            
        if let Some(o) = order {
            Ok(Response::new(SalesOrderDetail {
                order_number: o.order_number.clone(),
                header: Some(SalesOrderHeader {
                    sales_org: o.sales_org,
                    distribution_channel: o.distribution_channel,
                    division: o.division,
                    sold_to_party: o.sold_to_party,
                    document_date: Some(to_timestamp(o.document_date.and_hms_opt(0,0,0).unwrap().and_utc())),
                    // ... map other fields
                    ..Default::default()
                }),
                items: o.items.into_iter().map(|i| SalesOrderItem {
                    item_number: i.item_number,
                    material: i.material,
                    order_quantity: i.order_quantity.to_string(),
                    net_value: i.net_value.to_string(),
                    ..Default::default()
                }).collect(),
                schedule_lines: vec![], // TODO map
                audit_data: None,
                overall_status: common_v1::OrderStatus::Open.into(), // Map string to enum properly
            }))
        } else {
             Err(Status::not_found("Sales Order not found"))
        }

    }

    async fn list_sales_orders(
        &self,
        request: Request<ListSalesOrdersRequest>,
    ) -> Result<Response<ListSalesOrdersResponse>, Status> {
        let req = request.into_inner();
        let orders = self.list_handler.handle(ListSalesOrdersQuery {
            sold_to_party: if req.sold_to_party.is_empty() { None } else { Some(req.sold_to_party) },
            limit: 20,
        }).await.map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListSalesOrdersResponse {
            orders: orders.into_iter().map(|o| SalesOrderSummary {
                order_number: o.order_number,
                sold_to_party: o.sold_to_party,
                document_date: Some(to_timestamp(o.document_date.and_hms_opt(0,0,0).unwrap().and_utc())),
                net_value: Some(to_proto_money(o.net_value, &o.currency)),
                ..Default::default()
            }).collect(),
            pagination: None,
        }))
    }
    
    // Stubs
    async fn update_sales_order(&self, _r: Request<UpdateSalesOrderRequest>) -> Result<Response<SalesOrderResponse>, Status> { Err(Status::unimplemented("")) }
    async fn cancel_sales_order(&self, _r: Request<CancelSalesOrderRequest>) -> Result<Response<SalesOrderResponse>, Status> { Err(Status::unimplemented("")) }
    async fn get_sales_order_fulfillment_status(&self, _r: Request<GetSalesOrderRequest>) -> Result<Response<FulfillmentStatusResponse>, Status> { Err(Status::unimplemented("")) }
}
