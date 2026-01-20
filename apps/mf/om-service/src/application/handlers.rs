use crate::application::commands::{
    CreateOrderCommand, PostComponentsCommand, ReceiveGoodsCommand,
};
use crate::domain::{SubcontractingComponent, SubcontractingItem, SubcontractingOrder};
use crate::infrastructure::repository::SubcontractingRepository;
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct SubcontractingHandler {
    repo: Arc<SubcontractingRepository>,
}

impl SubcontractingHandler {
    pub fn new(repo: Arc<SubcontractingRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_order(&self, cmd: CreateOrderCommand) -> Result<String> {
        let order_id = Uuid::new_v4();
        let po_number = format!("SC{}", Utc::now().timestamp_subsec_micros());

        let items = cmd
            .items
            .into_iter()
            .enumerate()
            .map(|(idx, i)| {
                let item_id = Uuid::new_v4();
                SubcontractingItem {
                    item_id,
                    order_id,
                    item_number: (idx as i32 + 1) * 10,
                    finished_good_material: i.material,
                    order_quantity: Some(i.quantity),
                    received_quantity: rust_decimal::Decimal::ZERO,
                    unit: "EA".to_string(),
                    plant: i.plant,
                    components: i
                        .components
                        .into_iter()
                        .map(|c| SubcontractingComponent {
                            component_id: Uuid::new_v4(),
                            item_id,
                            component_material: c.material,
                            required_quantity: Some(c.quantity),
                            issued_quantity: rust_decimal::Decimal::ZERO,
                            unit: "EA".to_string(),
                        })
                        .collect(),
                }
            })
            .collect();

        let order = SubcontractingOrder {
            order_id,
            purchase_order_number: po_number.clone(),
            supplier: cmd.supplier,
            company_code: cmd.company_code,
            purchasing_org: cmd.purchasing_org,
            purchasing_group: None,
            created_at: Utc::now(),
            items,
        };

        self.repo.save(&order).await?;
        Ok(po_number)
    }

    pub async fn post_components(&self, _cmd: PostComponentsCommand) -> Result<String> {
        Ok(format!("MD{}", Utc::now().timestamp_subsec_micros()))
    }

    pub async fn receive_goods(&self, _cmd: ReceiveGoodsCommand) -> Result<String> {
        Ok(format!("MD{}", Utc::now().timestamp_subsec_micros()))
    }
}
