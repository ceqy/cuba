use crate::application::commands::{CreateBudgetCommand, PostDirectCostCommand};
use crate::domain::{CostPosting, ProjectBudget};
use crate::infrastructure::repository::ProjectCostRepository;
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct ProjectCostHandler {
    repo: Arc<ProjectCostRepository>,
}

impl ProjectCostHandler {
    pub fn new(repo: Arc<ProjectCostRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_budget(&self, cmd: CreateBudgetCommand) -> Result<String> {
        let budget_id = Uuid::new_v4();
        let b = ProjectBudget {
            budget_id,
            wbs_element: cmd.wbs_element,
            fiscal_year: cmd.fiscal_year,
            budget_amount: cmd.amount,
            currency: "CNY".to_string(),
            version: "ORIGINAL".to_string(),
            created_at: Utc::now(),
        };
        self.repo.save_budget(&b).await?;
        Ok(budget_id.to_string())
    }

    pub async fn post_direct_cost(&self, cmd: PostDirectCostCommand) -> Result<String> {
        let posting_id = Uuid::new_v4();
        let doc_num = format!("DOC{}", Utc::now().timestamp_subsec_micros());
        let p = CostPosting {
            posting_id,
            wbs_element: cmd.wbs_element,
            cost_element: cmd.cost_element,
            cost_element_type: "PRIMARY".to_string(),
            amount: cmd.amount,
            currency: "CNY".to_string(),
            posting_date: cmd.posting_date,
            description: cmd.description,
            document_number: Some(doc_num.clone()),
        };
        self.repo.save_posting(&p).await?;
        Ok(doc_num)
    }
}
