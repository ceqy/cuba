use crate::application::commands::{CreateContractCommand, RunPostingCommand};
use crate::domain::{PerformanceObligation, RevenueContract, RevenuePostingDocument};
use crate::infrastructure::repository::RevenueRepository;
use anyhow::Result;
use chrono::Utc;
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

pub struct RevenueHandler {
    repo: Arc<RevenueRepository>,
}

impl RevenueHandler {
    pub fn new(repo: Arc<RevenueRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_contract(&self, cmd: CreateContractCommand) -> Result<String> {
        let contract_id = Uuid::new_v4();
        let contract_number = format!("RC{}", Utc::now().timestamp_subsec_micros());

        // Create a default POB
        let pob_id = Uuid::new_v4();
        let pob_code = format!("POB-{}", Utc::now().timestamp_subsec_micros());

        let c = RevenueContract {
            contract_id,
            contract_number: contract_number.clone(),
            source_document_number: cmd.source_document_number,
            source_document_type: cmd.source_document_type,
            company_code: cmd.company_code,
            customer: cmd.customer,
            created_at: Utc::now(),
            obligations: vec![PerformanceObligation {
                pob_id,
                contract_id,
                pob_code,
                description: Some("Default POB".to_string()),
                allocated_price: Some(Decimal::new(10000, 0)),
                currency: "CNY".to_string(),
                recognition_method: "POINT_IN_TIME".to_string(),
                recognized_revenue: Decimal::ZERO,
                deferred_revenue: Decimal::new(10000, 0),
            }],
        };
        self.repo.save_contract(&c).await?;
        Ok(contract_number)
    }

    pub async fn run_posting(&self, cmd: RunPostingCommand) -> Result<(String, i32)> {
        let run_id = format!("RUN-{}", Utc::now().timestamp_subsec_micros());

        // Simplified: Create one posting document
        let posting = RevenuePostingDocument {
            posting_id: Uuid::new_v4(),
            document_id: format!("RVP{}", Utc::now().timestamp_subsec_micros()),
            posting_date: Utc::now().date_naive(),
            pob_id: Uuid::new_v4(), // Would lookup actual POBs
            amount: Decimal::new(5000, 0),
            currency: "CNY".to_string(),
            posting_type: Some("RECOGNITION".to_string()),
            accounting_document_number: None,
            created_at: Utc::now(),
        };
        self.repo.save_posting(&posting).await?;

        Ok((run_id, 1))
    }
}
