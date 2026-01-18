use sqlx::PgPool;
use crate::domain::{AllocationRun, AllocationSender, AllocationReceiver};
use anyhow::Result;

cuba_database::define_repository!(AllocationRepository);

impl AllocationRepository {

    pub async fn save_run(&self, run: &AllocationRun) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO allocation_runs (run_id, controlling_area, fiscal_year, fiscal_period, allocation_cycle, allocation_type, test_run, status) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
            .bind(run.run_id)
            .bind(&run.controlling_area)
            .bind(run.fiscal_year)
            .bind(run.fiscal_period)
            .bind(&run.allocation_cycle)
            .bind(&run.allocation_type)
            .bind(run.test_run)
            .bind(&run.status)
        .execute(&mut *tx).await?;

        for s in &run.senders {
            sqlx::query(
                "INSERT INTO allocation_senders (sender_id, run_id, sender_object, sent_amount, currency, cost_center, profit_center, segment, internal_order, wbs_element) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)")
                .bind(s.sender_id)
                .bind(s.run_id)
                .bind(&s.sender_object)
                .bind(s.sent_amount)
                .bind(&s.currency)
                .bind(&s.cost_center)
                .bind(&s.profit_center)
                .bind(&s.segment)
                .bind(&s.internal_order)
                .bind(&s.wbs_element)
            .execute(&mut *tx).await?;
        }

        for r in &run.receivers {
            sqlx::query(
                "INSERT INTO allocation_receivers (receiver_id, run_id, receiver_object, received_amount, currency, cost_center, profit_center, segment, internal_order, wbs_element) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)")
                .bind(r.receiver_id)
                .bind(r.run_id)
                .bind(&r.receiver_object)
                .bind(r.received_amount)
                .bind(&r.currency)
                .bind(&r.cost_center)
                .bind(&r.profit_center)
                .bind(&r.segment)
                .bind(&r.internal_order)
                .bind(&r.wbs_element)
            .execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_id(&self, run_id: uuid::Uuid) -> Result<Option<AllocationRun>> {
        let h = sqlx::query_as::<_, AllocationRun>(
            "SELECT run_id, controlling_area, fiscal_year, fiscal_period, allocation_cycle, allocation_type, test_run, status, created_at FROM allocation_runs WHERE run_id = $1")
            .bind(run_id)
            .fetch_optional(&self.pool).await?;
        
        if let Some(mut h) = h {
            let senders = sqlx::query_as::<_, AllocationSender>("SELECT * FROM allocation_senders WHERE run_id = $1")
                .bind(run_id)
                .fetch_all(&self.pool).await?;
            let receivers = sqlx::query_as::<_, AllocationReceiver>("SELECT * FROM allocation_receivers WHERE run_id = $1")
                .bind(run_id)
                .fetch_all(&self.pool).await?;
            h.senders = senders;
            h.receivers = receivers;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }
}
