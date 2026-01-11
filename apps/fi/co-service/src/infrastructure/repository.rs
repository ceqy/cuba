use sqlx::PgPool;
use crate::domain::{AllocationRun, AllocationSender, AllocationReceiver};
use anyhow::Result;

pub struct AllocationRepository {
    pool: PgPool,
}

impl AllocationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_run(&self, run: &AllocationRun) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            "INSERT INTO allocation_runs (run_id, controlling_area, fiscal_year, fiscal_period, allocation_cycle, allocation_type, test_run, status) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            run.run_id, run.controlling_area, run.fiscal_year, run.fiscal_period, run.allocation_cycle, run.allocation_type, run.test_run, run.status
        ).execute(&mut *tx).await?;

        for s in &run.senders {
            sqlx::query!(
                "INSERT INTO allocation_senders (sender_id, run_id, sender_object, sent_amount, currency) VALUES ($1, $2, $3, $4, $5)",
                s.sender_id, s.run_id, s.sender_object, s.sent_amount, s.currency
            ).execute(&mut *tx).await?;
        }

        for r in &run.receivers {
            sqlx::query!(
                "INSERT INTO allocation_receivers (receiver_id, run_id, receiver_object, received_amount, currency) VALUES ($1, $2, $3, $4, $5)",
                r.receiver_id, r.run_id, r.receiver_object, r.received_amount, r.currency
            ).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_id(&self, run_id: uuid::Uuid) -> Result<Option<AllocationRun>> {
        let h = sqlx::query!("SELECT * FROM allocation_runs WHERE run_id = $1", run_id)
            .fetch_optional(&self.pool).await?;
        if let Some(h) = h {
            let senders = sqlx::query!("SELECT * FROM allocation_senders WHERE run_id = $1", run_id)
                .fetch_all(&self.pool).await?;
            let receivers = sqlx::query!("SELECT * FROM allocation_receivers WHERE run_id = $1", run_id)
                .fetch_all(&self.pool).await?;
            Ok(Some(AllocationRun {
                run_id: h.run_id,
                controlling_area: h.controlling_area,
                fiscal_year: h.fiscal_year,
                fiscal_period: h.fiscal_period,
                allocation_cycle: h.allocation_cycle,
                allocation_type: h.allocation_type,
                test_run: h.test_run.unwrap_or(false),
                status: h.status.unwrap_or_else(|| "COMPLETED".to_string()),
                created_at: h.created_at,
                senders: senders.into_iter().map(|s| AllocationSender {
                    sender_id: s.sender_id,
                    run_id: s.run_id,
                    sender_object: s.sender_object,
                    sent_amount: s.sent_amount,
                    currency: s.currency.unwrap_or_else(|| "CNY".to_string()),
                }).collect(),
                receivers: receivers.into_iter().map(|r| AllocationReceiver {
                    receiver_id: r.receiver_id,
                    run_id: r.run_id,
                    receiver_object: r.receiver_object,
                    received_amount: r.received_amount,
                    currency: r.currency.unwrap_or_else(|| "CNY".to_string()),
                }).collect(),
            }))
        } else {
            Ok(None)
        }
    }
}
