use crate::application::commands::{
    ApproveContractCommand, ContractItemInput, CreateContractCommand,
};
use crate::application::handlers::ContractHandler;
use crate::infrastructure::repository::ContractRepository;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::api::proto::common::v1 as common_v1;
use crate::api::proto::pm::ct::v1 as ct_v1;

use ct_v1::contract_management_service_server::ContractManagementService;
use ct_v1::*;

pub struct CtServiceImpl {
    handler: Arc<ContractHandler>,
    repo: Arc<ContractRepository>,
}

impl CtServiceImpl {
    pub fn new(handler: Arc<ContractHandler>, repo: Arc<ContractRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl ContractManagementService for CtServiceImpl {
    async fn create_contract(
        &self,
        request: Request<CreateContractRequest>,
    ) -> Result<Response<ContractResponse>, Status> {
        let req = request.into_inner();
        let header = req.header.unwrap_or_default();
        let cmd = CreateContractCommand {
            company_code: header.company_code,
            supplier: header.supplier,
            purchasing_org: header.purchasing_org,
            validity_start: None,
            validity_end: None,
            target_value: header
                .target_value
                .map(|v| Decimal::from_str(&v.value).unwrap_or_default()),
            items: req
                .items
                .into_iter()
                .map(|i| ContractItemInput {
                    item_number: i.item_number,
                    material: if i.material.is_empty() {
                        None
                    } else {
                        Some(i.material)
                    },
                    short_text: if i.short_text.is_empty() {
                        None
                    } else {
                        Some(i.short_text)
                    },
                    target_quantity: i
                        .target_quantity
                        .map(|q| Decimal::from_str(&q.value).unwrap_or_default()),
                    net_price: i
                        .net_price
                        .map(|p| Decimal::from_str(&p.value).unwrap_or_default()),
                    plant: if i.plant.is_empty() {
                        None
                    } else {
                        Some(i.plant)
                    },
                })
                .collect(),
        };
        let num = self
            .handler
            .create_contract(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ContractResponse {
            success: true,
            contract_number: num,
            messages: vec![],
        }))
    }

    async fn get_contract(
        &self,
        request: Request<GetContractRequest>,
    ) -> Result<Response<ContractDetail>, Status> {
        let req = request.into_inner();
        let c = self
            .repo
            .find_by_number(&req.contract_number)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Contract not found"))?;
        Ok(Response::new(ContractDetail {
            contract_number: c.contract_number,
            header: Some(ContractHeader {
                company_code: c.company_code,
                document_type: common_v1::DocumentType::PurchaseOrder as i32,
                supplier: c.supplier,
                purchasing_org: c.purchasing_org,
                purchasing_group: c.purchasing_group.unwrap_or_default(),
                validity_start: None,
                validity_end: None,
                target_value: c.target_value.map(|v| common_v1::MonetaryValue {
                    value: v.to_string(),
                    currency_code: c.currency.clone(),
                }),
                release_status: common_v1::ReleaseStatus::NotReleased as i32,
            }),
            items: c
                .items
                .into_iter()
                .map(|i| ContractItem {
                    item_number: i.item_number,
                    material: i.material.unwrap_or_default(),
                    short_text: i.short_text.unwrap_or_default(),
                    target_quantity: i.target_quantity.map(|q| common_v1::QuantityValue {
                        value: q.to_string(),
                        unit_code: i.unit.clone(),
                    }),
                    unit: i.unit,
                    net_price: i.net_price.map(|p| common_v1::MonetaryValue {
                        value: p.to_string(),
                        currency_code: i.price_currency,
                    }),
                    price_unit: 1,
                    plant: i.plant.unwrap_or_default(),
                    item_category: common_v1::ItemCategory::Standard as i32,
                })
                .collect(),
            audit_data: None,
        }))
    }

    async fn approve_contract(
        &self,
        request: Request<ApproveContractRequest>,
    ) -> Result<Response<ContractResponse>, Status> {
        let req = request.into_inner();
        let cmd = ApproveContractCommand {
            contract_number: req.contract_number.clone(),
            approved: req.approved,
        };
        self.handler
            .approve_contract(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ContractResponse {
            success: true,
            contract_number: req.contract_number,
            messages: vec![],
        }))
    }

    async fn update_contract(
        &self,
        _r: Request<UpdateContractRequest>,
    ) -> Result<Response<ContractResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn list_contracts(
        &self,
        _r: Request<ListContractsRequest>,
    ) -> Result<Response<ListContractsResponse>, Status> {
        Err(Status::unimplemented(""))
    }
}
