use crate::application::commands::{
    CalculatePriceCommand, PricingItemInput, UpdateConditionCommand,
};
use crate::application::handlers::PricingHandler;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::api::proto::common::v1 as common_v1;
use crate::api::proto::sd::pe::v1 as pe_v1;

use pe_v1::pricing_engine_service_server::PricingEngineService;
use pe_v1::*;

pub struct PeServiceImpl {
    handler: Arc<PricingHandler>,
}

impl PeServiceImpl {
    pub fn new(handler: Arc<PricingHandler>) -> Self {
        Self { handler }
    }
}

#[tonic::async_trait]
impl PricingEngineService for PeServiceImpl {
    async fn calculate_price(
        &self,
        request: Request<CalculatePriceRequest>,
    ) -> Result<Response<CalculatePriceResponse>, Status> {
        let req = request.into_inner();
        let ctx = req.context.unwrap_or_default();
        let cmd = CalculatePriceCommand {
            sales_org: ctx.sales_org,
            customer: if ctx.customer.is_empty() {
                None
            } else {
                Some(ctx.customer)
            },
            pricing_date: chrono::Utc::now().date_naive(),
            items: req
                .items
                .into_iter()
                .map(|i| PricingItemInput {
                    item_id: i.item_id,
                    material: i.material,
                    quantity: Decimal::from_str(&i.quantity).unwrap_or_else(|_| Decimal::new(1, 0)),
                })
                .collect(),
        };
        let results = self
            .handler
            .calculate_price(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(CalculatePriceResponse {
            results: results
                .into_iter()
                .map(|r| PricingResult {
                    item_id: r.item_id,
                    net_price: Some(common_v1::MonetaryValue {
                        value: r.net_price.to_string(),
                        currency_code: "CNY".to_string(),
                    }),
                    tax_amount: Some(common_v1::MonetaryValue {
                        value: r.tax_amount.to_string(),
                        currency_code: "CNY".to_string(),
                    }),
                    gross_price: Some(common_v1::MonetaryValue {
                        value: r.gross_price.to_string(),
                        currency_code: "CNY".to_string(),
                    }),
                    conditions: r
                        .conditions
                        .into_iter()
                        .map(|c| PricingCondition {
                            condition_type: common_v1::PricingConditionType::Price as i32,
                            condition_value: c.value.to_string(),
                            currency_or_unit: c.currency,
                            description: c.description,
                        })
                        .collect(),
                })
                .collect(),
            messages: vec![],
        }))
    }

    async fn update_pricing_condition(
        &self,
        request: Request<UpdatePricingConditionRequest>,
    ) -> Result<Response<UpdatePricingConditionResponse>, Status> {
        let req = request.into_inner();
        let cmd = UpdateConditionCommand {
            condition_type: format!("{}", req.condition_type),
            material: if req.material.is_empty() {
                None
            } else {
                Some(req.material)
            },
            customer: if req.customer.is_empty() {
                None
            } else {
                Some(req.customer)
            },
            sales_org: req.sales_org,
            amount: Decimal::from_str(&req.amount).unwrap_or_default(),
            currency: req.currency,
            valid_from: None,
            valid_to: None,
        };
        let id = self
            .handler
            .update_condition(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(UpdatePricingConditionResponse {
            success: true,
            condition_record_id: id,
            messages: vec![],
        }))
    }
}
