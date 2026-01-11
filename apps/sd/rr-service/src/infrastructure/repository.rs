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
        sqlx::query!(
            "INSERT INTO revenue_contracts (contract_id, contract_number, source_document_number, source_document_type, company_code, customer) VALUES ($1, $2, $3, $4, $5, $6)",
            c.contract_id, c.contract_number, c.source_document_number, c.source_document_type, c.company_code, c.customer
        ).execute(&mut *tx).await?;

        for pob in &c.obligations {
            sqlx::query!(
                "INSERT INTO performance_obligations (pob_id, contract_id, pob_code, description, allocated_price, currency, recognition_method) VALUES ($1, $2, $3, $4, $5, $6, $7)",
                pob.pob_id, pob.contract_id, pob.pob_code, pob.description, pob.allocated_price, pob.currency, pob.recognition_method
            ).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_contract_by_number(&self, num: &str) -> Result<Option<RevenueContract>> {
        let h = sqlx::query!("SELECT * FROM revenue_contracts WHERE contract_number = $1", num)
            .fetch_optional(&self.pool).await?;
        if let Some(h) = h {
            let pobs = sqlx::query!("SELECT * FROM performance_obligations WHERE contract_id = $1", h.contract_id)
                .fetch_all(&self.pool).await?;
            Ok(Some(RevenueContract {
                contract_id: h.contract_id,
                contract_number: h.contract_number,
                source_document_number: h.source_document_number,
                source_document_type: h.source_document_type.unwrap_or_default(),
                company_code: h.company_code,
                customer: h.customer,
                created_at: h.created_at,
                obligations: pobs.into_iter().map(|p| PerformanceObligation {
                    pob_id: p.pob_id,
                    contract_id: p.contract_id,
                    pob_code: p.pob_code,
                    description: p.description,
                    allocated_price: p.allocated_price,
                    currency: p.currency.unwrap_or_default(),
                    recognition_method: p.recognition_method.unwrap_or_default(),
                    recognized_revenue: p.recognized_revenue.unwrap_or_default(),
                    deferred_revenue: p.deferred_revenue.unwrap_or_default(),
                }).collect(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn save_posting(&self, p: &RevenuePostingDocument) -> Result<()> {
        sqlx::query!(
            "INSERT INTO revenue_postings (posting_id, document_id, posting_date, pob_id, amount, currency, posting_type, accounting_document_number) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            p.posting_id, p.document_id, p.posting_date, p.pob_id, p.amount, p.currency, p.posting_type, p.accounting_document_number
        ).execute(&self.pool).await?;
        Ok(())
    }
}
