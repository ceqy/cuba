use std::sync::Arc;
use crate::domain::{ServiceContract, BillingPlanItem};
use crate::infrastructure::repository::ContractRepository;
use crate::application::commands::{CreateBillingPlanCommand, RunBillingCommand};
use anyhow::Result;
use uuid::Uuid;
use chrono::Utc;

pub struct BillingHandler {
    repo: Arc<ContractRepository>,
}

impl BillingHandler {
    pub fn new(repo: Arc<ContractRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_billing_plan(&self, cmd: CreateBillingPlanCommand) -> Result<String> {
        let contract_id = Uuid::new_v4();
        let c = ServiceContract {
            contract_id,
            contract_number: cmd.contract_number.clone(),
            customer_id: cmd.customer_id,
            validity_start: cmd.validity_start,
            validity_end: cmd.validity_end,
            created_at: Utc::now(),
            billing_plan: cmd.items.into_iter().map(|i| BillingPlanItem {
                item_id: Uuid::new_v4(),
                contract_id,
                planned_date: i.planned_date,
                amount: i.amount,
                currency: "CNY".to_string(),
                status: "OPEN".to_string(),
                invoice_number: None,
            }).collect(),
        };
        self.repo.create_contract(&c).await?;
        Ok(cmd.contract_number)
    }

    pub async fn run_billing(&self, cmd: RunBillingCommand) -> Result<i32> {
        self.repo.run_billing(cmd.until_date).await
    }
}
