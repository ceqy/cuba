use crate::domain::{AdjudicationResult, Claim};
use anyhow::Result;
use sqlx::PgPool;

pub struct ClaimRepository {
    pool: PgPool,
}

impl ClaimRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, c: &Claim) -> Result<()> {
        sqlx::query(
            "INSERT INTO claims (claim_id, customer_id, product_id, failure_date, failure_description, claimed_amount, currency, status, attachment_urls) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
            .bind(c.claim_id)
            .bind(&c.customer_id)
            .bind(&c.product_id)
            .bind(c.failure_date)
            .bind(&c.failure_description)
            .bind(c.claimed_amount)
            .bind(&c.currency)
            .bind(&c.status)
            .bind(&c.attachment_urls)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<Claim>> {
        let h = sqlx::query_as::<_, Claim>("SELECT claim_id, customer_id, product_id, failure_date, failure_description, claimed_amount, currency, status, attachment_urls, created_at FROM claims WHERE claim_id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await?;
        if let Some(mut h) = h {
            let adj = sqlx::query_as::<_, AdjudicationResult>("SELECT adjudication_id, claim_id, adjudicated_by, adjudication_date, approved_amount, currency, notes FROM adjudications WHERE claim_id = $1")
                .bind(id)
                .fetch_optional(&self.pool).await?;
            h.adjudication = adj;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }

    pub async fn adjudicate(
        &self,
        claim_id: uuid::Uuid,
        adj: &AdjudicationResult,
        new_status: &str,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("UPDATE claims SET status = $1 WHERE claim_id = $2")
            .bind(new_status)
            .bind(claim_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "INSERT INTO adjudications (adjudication_id, claim_id, adjudicated_by, approved_amount, currency, notes) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(adj.adjudication_id)
            .bind(adj.claim_id)
            .bind(&adj.adjudicated_by)
            .bind(adj.approved_amount)
            .bind(&adj.currency)
            .bind(&adj.notes)
        .execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }
}
