use sqlx::PgPool;
use crate::domain::{RevenueContract, PerformanceObligation, RevenuePostingDocument};
use anyhow::Result;

pub struct RevenueRepository {
    pool: PgPool,
}

impl RevenueRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_contract(&self, c: &RevenueContract) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO revenue_contracts (contract_id, contract_number, source_document_number, source_document_type, company_code, customer) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(c.contract_id)
            .bind(&c.contract_number)
            .bind(&c.source_document_number)
            .bind(&c.source_document_type)
            .bind(&c.company_code)
            .bind(&c.customer)
        .execute(&mut *tx).await?;

        for pob in &c.obligations {
            sqlx::query(
                "INSERT INTO performance_obligations (pob_id, contract_id, pob_code, description, allocated_price, currency, recognition_method) VALUES ($1, $2, $3, $4, $5, $6, $7)")
                .bind(pob.pob_id)
                .bind(pob.contract_id)
                .bind(&pob.pob_code)
                .bind(&pob.description)
                .bind(pob.allocated_price)
                .bind(&pob.currency)
                .bind(&pob.recognition_method)
            .execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_contract_by_number(&self, num: &str) -> Result<Option<RevenueContract>> {
        let h = sqlx::query_as::<_, RevenueContract>("SELECT contract_id, contract_number, source_document_number, COALESCE(source_document_type, '') as source_document_type, company_code, customer, created_at FROM revenue_contracts WHERE contract_number = $1")
            .bind(num)
            .fetch_optional(&self.pool).await?;
        if let Some(mut h) = h {
            let pobs = sqlx::query_as::<_, PerformanceObligation>("SELECT pob_id, contract_id, pob_code, description, allocated_price, COALESCE(currency, '') as currency, COALESCE(recognition_method, '') as recognition_method, COALESCE(recognized_revenue, 0) as recognized_revenue, COALESCE(deferred_revenue, 0) as deferred_revenue FROM performance_obligations WHERE contract_id = $1")
                .bind(h.contract_id)
                .fetch_all(&self.pool).await?;
            h.obligations = pobs;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }

    pub async fn save_posting(&self, p: &RevenuePostingDocument) -> Result<()> {
        sqlx::query(
            "INSERT INTO revenue_postings (posting_id, document_id, posting_date, pob_id, amount, currency, posting_type, accounting_document_number) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
            .bind(p.posting_id)
            .bind(&p.document_id)
            .bind(p.posting_date)
            .bind(p.pob_id)
            .bind(p.amount)
            .bind(&p.currency)
            .bind(&p.posting_type)
            .bind(&p.accounting_document_number)
        .execute(&self.pool).await?;
        Ok(())
    }
}
