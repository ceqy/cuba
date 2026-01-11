use std::sync::Arc;
use crate::domain::{AllocationRun, AllocationSender, AllocationReceiver};
use crate::infrastructure::repository::AllocationRepository;
use crate::application::commands::ExecuteAllocationCommand;
use anyhow::Result;
use uuid::Uuid;
use chrono::Utc;
use rust_decimal::Decimal;

pub struct AllocationHandler {
    repo: Arc<AllocationRepository>,
}

impl AllocationHandler {
    pub fn new(repo: Arc<AllocationRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute_allocation(&self, cmd: ExecuteAllocationCommand) -> Result<String> {
        let run_id = Uuid::new_v4();
        // Mock allocation: create sample senders and receivers
        let run = AllocationRun {
            run_id,
            controlling_area: cmd.controlling_area,
            fiscal_year: cmd.fiscal_year,
            fiscal_period: cmd.fiscal_period,
            allocation_cycle: cmd.allocation_cycle,
            allocation_type: cmd.allocation_type,
            test_run: cmd.test_run,
            status: if cmd.test_run { "TEST" } else { "COMPLETED" }.to_string(),
            created_at: Utc::now(),
            senders: vec![AllocationSender {
                sender_id: Uuid::new_v4(),
                run_id,
                sender_object: "CCTR-1000".to_string(),
                sent_amount: Decimal::new(10000, 2),
                currency: "CNY".to_string(),
            }],
            receivers: vec![AllocationReceiver {
                receiver_id: Uuid::new_v4(),
                run_id,
                receiver_object: "CCTR-2000".to_string(),
                received_amount: Decimal::new(10000, 2),
                currency: "CNY".to_string(),
            }],
        };
        self.repo.save_run(&run).await?;
        Ok(run_id.to_string())
    }
}
