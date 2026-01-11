use sqlx::PgPool;
use crate::domain::{Claim, AdjudicationResult};
use anyhow::Result;

pub struct ClaimRepository {
    pool: PgPool,
}

impl ClaimRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, c: &Claim) -> Result<()> {
        sqlx::query!(
            "INSERT INTO claims (claim_id, customer_id, product_id, failure_date, failure_description, claimed_amount, currency, status, attachment_urls) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            c.claim_id, c.customer_id, c.product_id, c.failure_date, c.failure_description, c.claimed_amount, c.currency, c.status, &c.attachment_urls
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<Claim>> {
        let h = sqlx::query!("SELECT * FROM claims WHERE claim_id = $1", id)
            .fetch_optional(&self.pool).await?;
        if let Some(h) = h {
            let adj = sqlx::query!("SELECT * FROM adjudications WHERE claim_id = $1", id)
                .fetch_optional(&self.pool).await?;
            Ok(Some(Claim {
                claim_id: h.claim_id,
                customer_id: h.customer_id,
                product_id: h.product_id,
                failure_date: h.failure_date,
                failure_description: h.failure_description,
                claimed_amount: h.claimed_amount,
                currency: h.currency.unwrap_or_default(),
                status: h.status.unwrap_or_default(),
                attachment_urls: h.attachment_urls.unwrap_or_default(),
                created_at: h.created_at,
                adjudication: adj.map(|a| AdjudicationResult {
                    adjudication_id: a.adjudication_id,
                    claim_id: a.claim_id,
                    adjudicated_by: a.adjudicated_by,
                    adjudication_date: a.adjudication_date,
                    approved_amount: a.approved_amount,
                    currency: a.currency.unwrap_or_default(),
                    notes: a.notes,
                }),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn adjudicate(&self, claim_id: uuid::Uuid, adj: &AdjudicationResult, new_status: &str) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!("UPDATE claims SET status = $1 WHERE claim_id = $2", new_status, claim_id)
            .execute(&mut *tx).await?;
        sqlx::query!(
            "INSERT INTO adjudications (adjudication_id, claim_id, adjudicated_by, approved_amount, currency, notes) VALUES ($1, $2, $3, $4, $5, $6)",
            adj.adjudication_id, adj.claim_id, adj.adjudicated_by, adj.approved_amount, adj.currency, adj.notes
        ).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }
}
