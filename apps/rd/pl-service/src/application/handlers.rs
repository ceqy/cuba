use crate::application::commands::SyncBOMCommand;
use crate::domain::{BOMItem, BillOfMaterial};
use crate::infrastructure::repository::BOMRepository;
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct PLMHandler {
    repo: Arc<BOMRepository>,
}

impl PLMHandler {
    pub fn new(repo: Arc<BOMRepository>) -> Self {
        Self { repo }
    }

    pub async fn sync_bom(&self, cmd: SyncBOMCommand) -> Result<String> {
        let bom_id = Uuid::new_v4();
        let bom = BillOfMaterial {
            bom_id,
            material: cmd.material,
            plant: cmd.plant,
            bom_usage: cmd.bom_usage,
            bom_status: "ACTIVE".to_string(),
            base_quantity: cmd.base_quantity,
            alternative_bom: "1".to_string(),
            valid_from: Utc::now().date_naive(),
            created_at: Utc::now(),
            items: cmd
                .items
                .into_iter()
                .enumerate()
                .map(|(idx, i)| BOMItem {
                    item_id: Uuid::new_v4(),
                    bom_id,
                    item_node: if i.item_node.is_empty() {
                        format!("{:04}", (idx + 1) * 10)
                    } else {
                        i.item_node
                    },
                    item_category: "L".to_string(),
                    component_material: i.component_material,
                    component_quantity: i.component_quantity,
                    component_unit: "EA".to_string(),
                    item_text: None,
                    recursive_allowed: false,
                })
                .collect(),
        };
        self.repo.sync_bom(&bom).await?;
        Ok(bom_id.to_string())
    }
}
