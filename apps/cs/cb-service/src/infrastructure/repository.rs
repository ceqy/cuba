use crate::domain::{BillingPlanItem, ServiceContract};
use anyhow::Result;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;

pub struct ContractRepository {
    pool: PgPool,
}

impl ContractRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_contract(&self, c: &ServiceContract) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO service_contracts (contract_id, contract_number, customer_id, validity_start, validity_end) VALUES ($1, $2, $3, $4, $5)")
            .bind(c.contract_id)
            .bind(&c.contract_number)
            .bind(&c.customer_id)
            .bind(c.validity_start)
            .bind(c.validity_end)
        .execute(&mut *tx).await?;

        for item in &c.billing_plan {
            sqlx::query(
                "INSERT INTO billing_plan_items (item_id, contract_id, planned_date, amount, currency, status) VALUES ($1, $2, $3, $4, $5, $6)")
                .bind(item.item_id)
                .bind(item.contract_id)
                .bind(item.planned_date)
                .bind(item.amount)
                .bind(&item.currency)
                .bind(&item.status)
            .execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_number(&self, contract_number: &str) -> Result<Option<ServiceContract>> {
        let h = sqlx::query_as::<_, ServiceContract>("SELECT contract_id, contract_number, customer_id, validity_start, validity_end, created_at FROM service_contracts WHERE contract_number = $1")
            .bind(contract_number)
            .fetch_optional(&self.pool).await?;
        if let Some(mut h) = h {
            let items = sqlx::query_as::<_, BillingPlanItem>(
                "SELECT * FROM billing_plan_items WHERE contract_id = $1 ORDER BY planned_date",
            )
            .bind(h.contract_id)
            .fetch_all(&self.pool)
            .await?;
            h.billing_plan = items;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }

    pub async fn run_billing(&self, until_date: NaiveDate) -> Result<i32> {
        let result = sqlx::query(
            "UPDATE billing_plan_items SET status = 'BILLED', invoice_number = 'INV' || item_id::text WHERE status = 'OPEN' AND planned_date <= $1")
            .bind(until_date)
        .execute(&self.pool).await?;
        Ok(result.rows_affected() as i32)
    }
}
