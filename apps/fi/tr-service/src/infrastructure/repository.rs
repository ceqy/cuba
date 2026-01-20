use crate::domain::{BankStatement, PaymentDocument, PaymentRun, StatementTransaction};
use anyhow::Result;
use sqlx::PgPool;

cuba_database::define_repository!(TreasuryRepository);

impl TreasuryRepository {
    pub async fn save_statement(&self, stmt: &BankStatement) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO bank_statements (statement_id, company_code, statement_format, status, house_bank, bank_account) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(stmt.statement_id)
            .bind(&stmt.company_code)
            .bind(&stmt.statement_format)
            .bind(&stmt.status)
            .bind(&stmt.house_bank)
            .bind(&stmt.bank_account)
        .execute(&mut *tx).await?;

        for t in &stmt.transactions {
            sqlx::query(
                "INSERT INTO statement_transactions (transaction_id, statement_id, value_date, amount, currency, memo, partner_name, transaction_type) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
                .bind(t.transaction_id)
                .bind(t.statement_id)
                .bind(t.value_date)
                .bind(t.amount)
                .bind(&t.currency)
                .bind(&t.memo)
                .bind(&t.partner_name)
                .bind(&t.transaction_type)
            .execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_statement_by_id(&self, id: uuid::Uuid) -> Result<Option<BankStatement>> {
        let h = sqlx::query_as::<_, BankStatement>(
            "SELECT statement_id, company_code, statement_format, status, house_bank, bank_account, created_at FROM bank_statements WHERE statement_id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await?;

        if let Some(mut h) = h {
            let txns = sqlx::query_as::<_, StatementTransaction>(
                "SELECT * FROM statement_transactions WHERE statement_id = $1",
            )
            .bind(id)
            .fetch_all(&self.pool)
            .await?;
            h.transactions = txns;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }

    /// List bank statements with optional company_code filter and pagination
    pub async fn list_statements(
        &self,
        company_code: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<BankStatement>> {
        let statements = if let Some(cc) = company_code {
            sqlx::query_as::<_, BankStatement>(
                "SELECT statement_id, company_code, statement_format, status, house_bank, bank_account, created_at 
                 FROM bank_statements 
                 WHERE company_code = $1 
                 ORDER BY created_at DESC 
                 LIMIT $2 OFFSET $3")
                .bind(cc)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool).await?
        } else {
            sqlx::query_as::<_, BankStatement>(
                "SELECT statement_id, company_code, statement_format, status, house_bank, bank_account, created_at 
                 FROM bank_statements 
                 ORDER BY created_at DESC 
                 LIMIT $1 OFFSET $2")
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool).await?
        };
        Ok(statements)
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
                "INSERT INTO payment_documents (doc_id, run_id, document_number, fiscal_year, amount, currency, payee_name, payment_method, house_bank, bank_account, transaction_type) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)")
                .bind(d.doc_id)
                .bind(d.run_id)
                .bind(&d.document_number)
                .bind(d.fiscal_year)
                .bind(d.amount)
                .bind(&d.currency)
                .bind(&d.payee_name)
                .bind(&d.payment_method)
                .bind(&d.house_bank)
                .bind(&d.bank_account)
                .bind(&d.transaction_type)
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
            let docs = sqlx::query_as::<_, PaymentDocument>(
                "SELECT * FROM payment_documents WHERE run_id = $1",
            )
            .bind(id)
            .fetch_all(&self.pool)
            .await?;
            h.documents = docs;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }

    /// List payment runs with optional status filter and pagination
    pub async fn list_payment_runs(
        &self,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PaymentRun>> {
        let runs = if let Some(s) = status {
            sqlx::query_as::<_, PaymentRun>(
                "SELECT run_id, run_number, company_codes, posting_date, status, created_at 
                 FROM payment_runs 
                 WHERE status = $1 
                 ORDER BY created_at DESC 
                 LIMIT $2 OFFSET $3",
            )
            .bind(s)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, PaymentRun>(
                "SELECT run_id, run_number, company_codes, posting_date, status, created_at 
                 FROM payment_runs 
                 ORDER BY created_at DESC 
                 LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };
        Ok(runs)
    }

    /// Update payment run status
    pub async fn update_run_status(&self, run_id: uuid::Uuid, status: &str) -> Result<()> {
        sqlx::query("UPDATE payment_runs SET status = $1 WHERE run_id = $2")
            .bind(status)
            .bind(run_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Count bank statements with optional company_code filter
    pub async fn count_statements(&self, company_code: Option<&str>) -> Result<i64> {
        let count: (i64,) = if let Some(cc) = company_code {
            sqlx::query_as("SELECT COUNT(*) FROM bank_statements WHERE company_code = $1")
                .bind(cc)
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query_as("SELECT COUNT(*) FROM bank_statements")
                .fetch_one(&self.pool)
                .await?
        };
        Ok(count.0)
    }
}
