use crate::application::commands::{AdjudicateClaimCommand, CreateClaimCommand};
use crate::domain::{AdjudicationResult, Claim};
use crate::infrastructure::repository::ClaimRepository;
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct ClaimHandler {
    repo: Arc<ClaimRepository>,
}

impl ClaimHandler {
    pub fn new(repo: Arc<ClaimRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_claim(&self, cmd: CreateClaimCommand) -> Result<String> {
        let claim_id = Uuid::new_v4();
        let c = Claim {
            claim_id,
            customer_id: cmd.customer_id,
            product_id: cmd.product_id,
            failure_date: cmd.failure_date,
            failure_description: cmd.failure_description,
            claimed_amount: cmd.claimed_amount,
            currency: "CNY".to_string(),
            status: "SUBMITTED".to_string(),
            attachment_urls: vec![],
            created_at: Utc::now(),
            adjudication: None,
        };
        self.repo.save(&c).await?;
        Ok(claim_id.to_string())
    }

    pub async fn adjudicate(&self, cmd: AdjudicateClaimCommand) -> Result<String> {
        let adj = AdjudicationResult {
            adjudication_id: Uuid::new_v4(),
            claim_id: cmd.claim_id,
            adjudicated_by: "SYSTEM".to_string(),
            adjudication_date: Utc::now(),
            approved_amount: cmd.approved_amount,
            currency: "CNY".to_string(),
            notes: cmd.notes,
        };
        self.repo
            .adjudicate(cmd.claim_id, &adj, &cmd.new_status)
            .await?;
        Ok(adj.adjudication_id.to_string())
    }
}
