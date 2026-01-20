use crate::domain::{Contract, ContractItem};
use anyhow::Result;
use sqlx::PgPool;

pub struct ContractRepository {
    pool: PgPool,
}

impl ContractRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, c: &Contract) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO contracts (contract_id, contract_number, company_code, supplier, purchasing_org, purchasing_group, validity_start, validity_end, target_value, currency, release_status) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)")
            .bind(c.contract_id)
            .bind(&c.contract_number)
            .bind(&c.company_code)
            .bind(&c.supplier)
            .bind(&c.purchasing_org)
            .bind(&c.purchasing_group)
            .bind(c.validity_start)
            .bind(c.validity_end)
            .bind(c.target_value)
            .bind(&c.currency)
            .bind(&c.release_status)
        .execute(&mut *tx).await?;

        for item in &c.items {
            sqlx::query(
                "INSERT INTO contract_items (item_id, contract_id, item_number, material, short_text, target_quantity, unit, net_price, price_currency, plant) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)")
                .bind(item.item_id)
                .bind(item.contract_id)
                .bind(item.item_number)
                .bind(&item.material)
                .bind(&item.short_text)
                .bind(item.target_quantity)
                .bind(&item.unit)
                .bind(item.net_price)
                .bind(&item.price_currency)
                .bind(&item.plant)
            .execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_number(&self, num: &str) -> Result<Option<Contract>> {
        let h = sqlx::query_as::<_, Contract>("SELECT contract_id, contract_number, company_code, supplier, purchasing_org, purchasing_group, validity_start, validity_end, target_value, currency, release_status, created_at FROM contracts WHERE contract_number = $1")
            .bind(num)
            .fetch_optional(&self.pool).await?;
        if let Some(mut h) = h {
            let items = sqlx::query_as::<_, ContractItem>(
                "SELECT * FROM contract_items WHERE contract_id = $1 ORDER BY item_number",
            )
            .bind(h.contract_id)
            .fetch_all(&self.pool)
            .await?;
            h.items = items;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }

    pub async fn approve(&self, contract_id: uuid::Uuid, approved: bool) -> Result<()> {
        let status = if approved { "RELEASED" } else { "REJECTED" };
        sqlx::query("UPDATE contracts SET release_status = $1 WHERE contract_id = $2")
            .bind(status)
            .bind(contract_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
