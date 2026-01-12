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
        sqlx::query(
            "INSERT INTO bank_statements (statement_id, company_code, statement_format, status) VALUES ($1, $2, $3, $4)")
            .bind(stmt.statement_id)
            .bind(&stmt.company_code)
            .bind(&stmt.statement_format)
            .bind(&stmt.status)
        .execute(&mut *tx).await?;

        for t in &stmt.transactions {
            sqlx::query(
                "INSERT INTO statement_transactions (transaction_id, statement_id, value_date, amount, currency, memo, partner_name) VALUES ($1, $2, $3, $4, $5, $6, $7)")
                .bind(t.transaction_id)
                .bind(t.statement_id)
                .bind(t.value_date)
                .bind(t.amount)
                .bind(&t.currency)
                .bind(&t.memo)
                .bind(&t.partner_name)
            .execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_statement_by_id(&self, id: uuid::Uuid) -> Result<Option<BankStatement>> {
        let h = sqlx::query_as::<_, BankStatement>(
            "SELECT statement_id, company_code, statement_format, status, created_at FROM bank_statements WHERE statement_id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await?;
        
        if let Some(mut h) = h {
            let txns = sqlx::query_as::<_, StatementTransaction>("SELECT * FROM statement_transactions WHERE statement_id = $1")
                .bind(id)
                .fetch_all(&self.pool).await?;
            h.transactions = txns;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }

    pub async fn save_payment_run(&self, run: &PaymentRun) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO payment_runs (run_id, run_number, company_codes, posting_date, status) VALUES ($1, $2, $3, $4, $5)")
            .bind(run.run_id)
            .bind(&run.run_number)
            .bind(&run.company_codes)
            .bind(run.posting_date)
            .bind(&run.status)
        .execute(&mut *tx).await?;

        for d in &run.documents {
            sqlx::query(
                "INSERT INTO payment_documents (doc_id, run_id, document_number, fiscal_year, amount, currency, payee_name) VALUES ($1, $2, $3, $4, $5, $6, $7)")
                .bind(d.doc_id)
                .bind(d.run_id)
                .bind(&d.document_number)
                .bind(d.fiscal_year)
                .bind(d.amount)
                .bind(&d.currency)
                .bind(&d.payee_name)
            .execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_run_by_id(&self, id: uuid::Uuid) -> Result<Option<PaymentRun>> {
        let h = sqlx::query_as::<_, PaymentRun>("SELECT run_id, run_number, company_codes, posting_date, status, created_at FROM payment_runs WHERE run_id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await?;
        
        if let Some(mut h) = h {
            let docs = sqlx::query_as::<_, PaymentDocument>("SELECT * FROM payment_documents WHERE run_id = $1")
                .bind(id)
                .fetch_all(&self.pool).await?;
            h.documents = docs;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }
}
