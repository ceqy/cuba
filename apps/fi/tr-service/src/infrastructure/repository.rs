use sqlx::PgPool;
use crate::domain::{BankStatement, StatementTransaction, PaymentRun, PaymentDocument};
use anyhow::Result;

pub struct TreasuryRepository {
    pool: PgPool,
}

impl TreasuryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_statement(&self, stmt: &BankStatement) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            "INSERT INTO bank_statements (statement_id, company_code, statement_format, status) VALUES ($1, $2, $3, $4)",
            stmt.statement_id, stmt.company_code, stmt.statement_format, stmt.status
        ).execute(&mut *tx).await?;

        for t in &stmt.transactions {
            sqlx::query!(
                "INSERT INTO statement_transactions (transaction_id, statement_id, value_date, amount, currency, memo, partner_name) VALUES ($1, $2, $3, $4, $5, $6, $7)",
                t.transaction_id, t.statement_id, t.value_date, t.amount, t.currency, t.memo, t.partner_name
            ).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_statement_by_id(&self, id: uuid::Uuid) -> Result<Option<BankStatement>> {
        let h = sqlx::query!("SELECT * FROM bank_statements WHERE statement_id = $1", id)
            .fetch_optional(&self.pool).await?;
        if let Some(h) = h {
            let txns = sqlx::query!("SELECT * FROM statement_transactions WHERE statement_id = $1", id)
                .fetch_all(&self.pool).await?;
            Ok(Some(BankStatement {
                statement_id: h.statement_id,
                company_code: h.company_code,
                statement_format: h.statement_format.unwrap_or_default(),
                status: h.status.unwrap_or_default(),
                created_at: h.created_at,
                transactions: txns.into_iter().map(|t| StatementTransaction {
                    transaction_id: t.transaction_id,
                    statement_id: t.statement_id,
                    value_date: t.value_date,
                    amount: t.amount,
                    currency: t.currency.unwrap_or_default(),
                    memo: t.memo,
                    partner_name: t.partner_name,
                }).collect(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn save_payment_run(&self, run: &PaymentRun) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            "INSERT INTO payment_runs (run_id, run_number, company_codes, posting_date, status) VALUES ($1, $2, $3, $4, $5)",
            run.run_id, run.run_number, run.company_codes, run.posting_date, run.status
        ).execute(&mut *tx).await?;

        for d in &run.documents {
            sqlx::query!(
                "INSERT INTO payment_documents (doc_id, run_id, document_number, fiscal_year, amount, currency, payee_name) VALUES ($1, $2, $3, $4, $5, $6, $7)",
                d.doc_id, d.run_id, d.document_number, d.fiscal_year, d.amount, d.currency, d.payee_name
            ).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_run_by_id(&self, id: uuid::Uuid) -> Result<Option<PaymentRun>> {
        let h = sqlx::query!("SELECT * FROM payment_runs WHERE run_id = $1", id)
            .fetch_optional(&self.pool).await?;
        if let Some(h) = h {
            let docs = sqlx::query!("SELECT * FROM payment_documents WHERE run_id = $1", id)
                .fetch_all(&self.pool).await?;
            Ok(Some(PaymentRun {
                run_id: h.run_id,
                run_number: h.run_number,
                company_codes: h.company_codes,
                posting_date: h.posting_date,
                status: h.status.unwrap_or_default(),
                created_at: h.created_at,
                documents: docs.into_iter().map(|d| PaymentDocument {
                    doc_id: d.doc_id,
                    run_id: d.run_id,
                    document_number: d.document_number,
                    fiscal_year: d.fiscal_year,
                    amount: d.amount,
                    currency: d.currency.unwrap_or_default(),
                    payee_name: d.payee_name,
                }).collect(),
            }))
        } else {
            Ok(None)
        }
    }
}
