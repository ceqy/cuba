use crate::application::commands::{ApproveContractCommand, CreateContractCommand};
use crate::domain::{Contract, ContractItem};
use crate::infrastructure::repository::ContractRepository;
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct ContractHandler {
    repo: Arc<ContractRepository>,
}

impl ContractHandler {
    pub fn new(repo: Arc<ContractRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_contract(&self, cmd: CreateContractCommand) -> Result<String> {
        let contract_id = Uuid::new_v4();
        let contract_number = format!("CT{}", Utc::now().timestamp_subsec_micros());
        let c = Contract {
            contract_id,
            contract_number: contract_number.clone(),
            company_code: cmd.company_code,
            supplier: cmd.supplier,
            purchasing_org: cmd.purchasing_org,
            purchasing_group: None,
            validity_start: cmd.validity_start,
            validity_end: cmd.validity_end,
            target_value: cmd.target_value,
            currency: "CNY".to_string(),
            release_status: "NOT_RELEASED".to_string(),
            created_at: Utc::now(),
            items: cmd
                .items
                .into_iter()
                .map(|i| ContractItem {
                    item_id: Uuid::new_v4(),
                    contract_id,
                    item_number: i.item_number,
                    material: i.material,
                    short_text: i.short_text,
                    target_quantity: i.target_quantity,
                    unit: "EA".to_string(),
                    net_price: i.net_price,
                    price_currency: "CNY".to_string(),
                    plant: i.plant,
                })
                .collect(),
        };
        self.repo.save(&c).await?;
        Ok(contract_number)
    }

    pub async fn approve_contract(&self, cmd: ApproveContractCommand) -> Result<()> {
        let c = self
            .repo
            .find_by_number(&cmd.contract_number)
            .await?
            .ok_or_else(|| anyhow!("Contract not found"))?;
        self.repo.approve(c.contract_id, cmd.approved).await
    }
}
