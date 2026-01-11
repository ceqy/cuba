use std::sync::Arc;
use crate::domain::{BankStatement, StatementTransaction, PaymentRun, PaymentDocument};
use crate::infrastructure::repository::TreasuryRepository;
use crate::application::commands::{ProcessStatementCommand, ExecutePaymentRunCommand};
use anyhow::Result;
use uuid::Uuid;
use chrono::Utc;
use rust_decimal::Decimal;

pub struct TreasuryHandler {
    repo: Arc<TreasuryRepository>,
}

impl TreasuryHandler {
    pub fn new(repo: Arc<TreasuryRepository>) -> Self {
        Self { repo }
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
        let run_number = format!("PR{}", Utc::now().timestamp_subsec_micros());
        let run = PaymentRun {
            run_id,
            run_number: run_number.clone(),
            company_codes: Some(cmd.company_codes.join(",")),
            posting_date: Some(Utc::now().date_naive()),
            status: "COMPLETED".to_string(),
            created_at: Utc::now(),
            documents: vec![PaymentDocument {
                doc_id: Uuid::new_v4(),
                run_id,
                document_number: format!("1500{}", Utc::now().timestamp_subsec_micros()),
                fiscal_year: Some(2026),
                amount: Decimal::new(5000, 2),
                currency: "CNY".to_string(),
                payee_name: Some("Vendor ABC".to_string()),
            }],
        };
        self.repo.save_payment_run(&run).await?;
        Ok(run_id.to_string())
    }
}
