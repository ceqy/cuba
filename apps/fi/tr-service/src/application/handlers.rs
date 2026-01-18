use std::sync::Arc;
use tokio::sync::Mutex;
use crate::domain::{BankStatement, StatementTransaction, PaymentRun, PaymentDocument};
use crate::infrastructure::repository::TreasuryRepository;
use cuba_finance::{GlClient, GlLineItem};
use crate::application::commands::{ProcessStatementCommand, ExecutePaymentRunCommand};
use anyhow::Result;
use uuid::Uuid;
use chrono::Utc;
use rust_decimal::Decimal;

pub struct TreasuryHandler {
    repo: Arc<TreasuryRepository>,
    gl_client: Arc<Mutex<GlClient>>,
}

impl TreasuryHandler {
    pub fn new(repo: Arc<TreasuryRepository>, gl_client: Arc<Mutex<GlClient>>) -> Self {
        Self { repo, gl_client }
    }

    pub async fn process_statement(&self, cmd: ProcessStatementCommand) -> Result<String> {
        let stmt_id = Uuid::new_v4();
        let stmt = BankStatement {
            statement_id: stmt_id,
            company_code: cmd.company_code,
            statement_format: "MT940".to_string(),
            status: "PROCESSED".to_string(),
            created_at: Utc::now(),
            transactions: vec![StatementTransaction {
                transaction_id: Uuid::new_v4(),
                statement_id: stmt_id,
                value_date: Utc::now().date_naive(),
                amount: Decimal::new(10000, 2),
                currency: "CNY".to_string(),
                memo: Some("Sample Transaction".to_string()),
                partner_name: Some("Test Partner".to_string()),
            }],
        };
        self.repo.save_statement(&stmt).await?;
        Ok(stmt_id.to_string())
    }

    pub async fn execute_payment_run(&self, cmd: ExecutePaymentRunCommand) -> Result<String> {
        let run_id = Uuid::new_v4();
        let now = Utc::now();
        let run_number = format!("PR{}", now.timestamp_subsec_micros());
        let payment_amount = Decimal::new(5000, 2);
        
        let run = PaymentRun {
            run_id,
            run_number: run_number.clone(),
            company_codes: Some(cmd.company_codes.join(",")),
            posting_date: Some(now.date_naive()),
            status: "COMPLETED".to_string(),
            created_at: now,
            documents: vec![PaymentDocument {
                doc_id: Uuid::new_v4(),
                run_id,
                document_number: format!("1500{}", now.timestamp_subsec_micros()),
                fiscal_year: Some(2026),
                amount: payment_amount,
                currency: "CNY".to_string(),
                payee_name: Some("Vendor ABC".to_string()),
            }],
        };
        
        // Save the payment run
        self.repo.save_payment_run(&run).await?;

        // Create GL journal entry for payment
        // Debit: AP Clearing (reduce liability)
        // Credit: Bank Account (cash outflow)
        let gl_line_items = vec![
            GlLineItem {
                gl_account: "211000".to_string(), // Accounts Payable
                debit_credit: "S".to_string(), // Debit (reduce payable)
                amount: payment_amount,
                cost_center: None,
                profit_center: None,
                item_text: Some(format!("Payment Run {} - Clear AP", run_number)),
                business_partner: Some("Vendor ABC".to_string()),
            },
            GlLineItem {
                gl_account: "113000".to_string(), // Bank Account
                debit_credit: "H".to_string(), // Credit (cash out)
                amount: payment_amount,
                cost_center: None,
                profit_center: None,
                item_text: Some(format!("Payment Run {} - Bank Outflow", run_number)),
                business_partner: None,
            },
        ];

        let mut gl_client = self.gl_client.lock().await;
        let posting_date = now.date_naive();
        let company_code = cmd.company_codes.first().cloned().unwrap_or_else(|| "1000".to_string());
        match gl_client.create_invoice_journal_entry(
            &company_code,
            posting_date,
            posting_date,
            2026,
            "CNY",
            Some(format!("TR-{}", run_id)),
            Some(format!("Payment Run {}", run_number)),
            gl_line_items,
        ).await {
            Ok(response) => {
                tracing::info!(
                    "GL Journal Entry created for payment run {}: {:?}",
                    run_id,
                    response.document_reference
                );
            }
            Err(e) => {
                tracing::error!("Failed to create GL entry for payment run {}: {}", run_id, e);
            }
        }

        Ok(run_id.to_string())
    }
}
