use sqlx::PgPool;
use crate::domain::{ServiceContract, BillingPlanItem};
use anyhow::Result;
use chrono::NaiveDate;
use rust_decimal::Decimal;

pub struct ContractRepository {
    pool: PgPool,
}

impl ContractRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_contract(&self, c: &ServiceContract) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            "INSERT INTO service_contracts (contract_id, contract_number, customer_id, validity_start, validity_end) VALUES ($1, $2, $3, $4, $5)",
            c.contract_id, c.contract_number, c.customer_id, c.validity_start, c.validity_end
        ).execute(&mut *tx).await?;

        for item in &c.billing_plan {
            sqlx::query!(
                "INSERT INTO billing_plan_items (item_id, contract_id, planned_date, amount, currency, status) VALUES ($1, $2, $3, $4, $5, $6)",
                item.item_id, item.contract_id, item.planned_date, item.amount, item.currency, item.status
            ).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_number(&self, contract_number: &str) -> Result<Option<ServiceContract>> {
        let h = sqlx::query!("SELECT * FROM service_contracts WHERE contract_number = $1", contract_number)
            .fetch_optional(&self.pool).await?;
        if let Some(h) = h {
            let items = sqlx::query!("SELECT * FROM billing_plan_items WHERE contract_id = $1 ORDER BY planned_date", h.contract_id)
                .fetch_all(&self.pool).await?;
            Ok(Some(ServiceContract {
                contract_id: h.contract_id,
                contract_number: h.contract_number,
                customer_id: h.customer_id,
                validity_start: h.validity_start,
                validity_end: h.validity_end,
                created_at: h.created_at,
                billing_plan: items.into_iter().map(|i| BillingPlanItem {
                    item_id: i.item_id,
                    contract_id: i.contract_id,
                    planned_date: i.planned_date,
                    amount: i.amount,
                    currency: i.currency.unwrap_or_else(|| "CNY".to_string()),
                    status: i.status.unwrap_or_else(|| "OPEN".to_string()),
                    invoice_number: i.invoice_number,
                }).collect(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn run_billing(&self, until_date: NaiveDate) -> Result<i32> {
        // Mark items as BILLED and generate invoice numbers
        let result = sqlx::query!(
            "UPDATE billing_plan_items SET status = 'BILLED', invoice_number = 'INV' || item_id::text WHERE status = 'OPEN' AND planned_date <= $1",
            until_date
        ).execute(&self.pool).await?;
        Ok(result.rows_affected() as i32)
    }
}
