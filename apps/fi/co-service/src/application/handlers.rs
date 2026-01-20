use crate::application::commands::ExecuteAllocationCommand;
use crate::domain::{AllocationReceiver, AllocationRun, AllocationSender};
use crate::infrastructure::repository::AllocationRepository;
use anyhow::Result;
use chrono::{Datelike, Utc};
use cuba_finance::{GlClient, GlLineItem};
use rust_decimal::Decimal;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct AllocationHandler {
    repo: Arc<AllocationRepository>,
    gl_client: Arc<Mutex<GlClient>>,
}

impl AllocationHandler {
    pub fn new(repo: Arc<AllocationRepository>, gl_client: Arc<Mutex<GlClient>>) -> Self {
        Self { repo, gl_client }
    }

    pub async fn execute_allocation(&self, cmd: ExecuteAllocationCommand) -> Result<String> {
        let run_id = Uuid::new_v4();
        let now = Utc::now();

        // Mock allocation: create sample senders and receivers
        let sender_amount = Decimal::new(10000, 2);
        let run = AllocationRun {
            run_id,
            controlling_area: cmd.controlling_area.clone(),
            fiscal_year: cmd.fiscal_year,
            fiscal_period: cmd.fiscal_period,
            allocation_cycle: cmd.allocation_cycle.clone(),
            allocation_type: cmd.allocation_type.clone(),
            test_run: cmd.test_run,
            status: if cmd.test_run { "TEST" } else { "COMPLETED" }.to_string(),
            created_at: now,
            senders: vec![AllocationSender {
                sender_id: Uuid::new_v4(),
                run_id,
                sender_object: "CCTR-1000".to_string(),
                sent_amount: sender_amount,
                currency: "CNY".to_string(),
                cost_center: Some("CCTR-1000".to_string()),
                profit_center: None,
                segment: None,
                internal_order: None,
                wbs_element: None,
            }],
            receivers: vec![AllocationReceiver {
                receiver_id: Uuid::new_v4(),
                run_id,
                receiver_object: "CCTR-2000".to_string(),
                received_amount: sender_amount,
                currency: "CNY".to_string(),
                cost_center: Some("CCTR-2000".to_string()),
                profit_center: None,
                segment: None,
                internal_order: None,
                wbs_element: None,
            }],
        };

        // Save allocation run
        self.repo.save_run(&run).await?;

        // Skip GL integration for test runs
        if cmd.test_run {
            return Ok(run_id.to_string());
        }

        // Create GL journal entry for cost allocation
        // Debit: Receiving cost center expense account
        // Credit: Sending cost center expense account
        let gl_line_items = vec![
            GlLineItem {
                gl_account: "6100".to_string(), // Receiving CC expense
                debit_credit: "S".to_string(),  // Debit
                amount: sender_amount,
                cost_center: Some("CCTR-2000".to_string()),
                profit_center: None,
                item_text: Some(format!(
                    "Cost allocation from {} cycle {}",
                    run.allocation_type, run.allocation_cycle
                )),
                business_partner: None,
                special_gl_indicator: None,
                ledger: None,
                ledger_type: None,
                financial_area: None,
                business_area: None,
                controlling_area: None,
            },
            GlLineItem {
                gl_account: "6100".to_string(), // Sending CC expense
                debit_credit: "H".to_string(),  // Credit
                amount: sender_amount,
                cost_center: Some("CCTR-1000".to_string()),
                profit_center: None,
                item_text: Some(format!(
                    "Cost allocation to {} cycle {}",
                    run.allocation_type, run.allocation_cycle
                )),
                business_partner: None,
                special_gl_indicator: None,
                ledger: None,
                ledger_type: None,
                financial_area: None,
                business_area: None,
                controlling_area: None,
            },
        ];

        let mut gl_client = self.gl_client.lock().await;
        let posting_date = now.date_naive();
        match gl_client
            .create_invoice_journal_entry(
                &cmd.controlling_area,
                posting_date,
                posting_date,
                cmd.fiscal_year,
                "CNY",
                Some(format!("CO-{}", run_id)),
                Some(format!("{} Allocation Run", cmd.allocation_type)),
                gl_line_items,
                None, // 使用默认主分类账 "0L"
            )
            .await
        {
            Ok(response) => {
                tracing::info!(
                    "GL Journal Entry created for allocation run {}: {:?}",
                    run_id,
                    response.document_reference
                );
            },
            Err(e) => {
                tracing::error!(
                    "Failed to create GL entry for allocation run {}: {}",
                    run_id,
                    e
                );
            },
        }

        Ok(run_id.to_string())
    }
}
