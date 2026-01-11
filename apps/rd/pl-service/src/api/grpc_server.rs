use tonic::{Request, Response, Status};
use std::sync::Arc;
use crate::application::commands::{SyncBOMCommand, BOMItemCmd};
use crate::application::handlers::PLMHandler;
use crate::infrastructure::repository::BOMRepository;
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::api::proto::rd::pl::v1 as pl_v1;
use crate::api::proto::common::v1 as common_v1;

use pl_v1::plm_integration_service_server::PlmIntegrationService;
use pl_v1::*;

pub struct PlServiceImpl {
    handler: Arc<PLMHandler>,
    repo: Arc<BOMRepository>,
}

impl PlServiceImpl {
    pub fn new(handler: Arc<PLMHandler>, repo: Arc<BOMRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl PlmIntegrationService for PlServiceImpl {

    async fn sync_bill_of_material(
        &self,
        request: Request<SyncBomRequest>,
    ) -> Result<Response<common_v1::JobInfo>, Status> {
        let req = request.into_inner();
        let bom = req.bom.unwrap_or_default();
        let header = bom.header.unwrap_or_default();
        let cmd = SyncBOMCommand {
            material: header.material,
            plant: header.plant,
            bom_usage: "PRODUCTION".to_string(),
            base_quantity: Decimal::from_str(&header.base_quantity).unwrap_or_else(|_| Decimal::ONE),
            items: bom.items.into_iter().map(|i| BOMItemCmd {
                item_node: i.item_node,
                component_material: i.component_material,
                component_quantity: Decimal::from_str(&i.component_quantity).unwrap_or_default(),
            }).collect(),
        };
        let bom_id = self.handler.sync_bom(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(common_v1::JobInfo {
            job_id: bom_id,
            job_type: "BOM_SYNC".to_string(),
            status: common_v1::JobStatus::Completed as i32,
            progress_percentage: 100,
            messages: vec![],
            error_detail: "".to_string(),
            created_at: None,
            started_at: None,
            completed_at: None,
        }))
    }

    async fn get_bill_of_material(
        &self,
        request: Request<GetBomRequest>,
    ) -> Result<Response<BillOfMaterial>, Status> {
        let req = request.into_inner();
        let usage = match req.bom_usage {
            1 => "PRODUCTION",
            2 => "ENGINEERING",
            _ => "PRODUCTION",
        };
        let bom = self.repo.find_by_key(&req.material, &req.plant, usage).await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("BOM not found"))?;
        Ok(Response::new(BillOfMaterial {
            header: Some(BomHeader {
                material: bom.material,
                plant: bom.plant,
                bom_usage: common_v1::BomUsage::Production as i32,
                bom_status: common_v1::BomStatus::Active as i32,
                base_quantity: bom.base_quantity.to_string(),
                alternative_bom: bom.alternative_bom,
                valid_from: None,
            }),
            items: bom.items.into_iter().map(|i| BomItem {
                item_node: i.item_node,
                item_category: common_v1::BomItemCategory::StockItem as i32,
                component_material: i.component_material,
                component_quantity: i.component_quantity.to_string(),
                component_unit: i.component_unit,
                item_text: i.item_text.unwrap_or_default(),
                recursive_allowed: i.recursive_allowed,
            }).collect(),
            audit_data: None,
        }))
    }

    // Stubs
    async fn sync_material_master(&self, _r: Request<SyncMaterialMasterRequest>) -> Result<Response<SyncMaterialMasterResponse>, Status> { Err(Status::unimplemented("")) }
    async fn update_bill_of_material(&self, _r: Request<UpdateBomRequest>) -> Result<Response<BomResponse>, Status> { Err(Status::unimplemented("")) }
    async fn list_bill_of_materials(&self, _r: Request<ListBoMsRequest>) -> Result<Response<ListBoMsResponse>, Status> { Err(Status::unimplemented("")) }
}
