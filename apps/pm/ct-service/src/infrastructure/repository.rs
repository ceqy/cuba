use sqlx::PgPool;
use crate::domain::{Contract, ContractItem};
use anyhow::Result;

pub struct ContractRepository {
    pool: PgPool,
}

impl ContractRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, c: &Contract) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            "INSERT INTO contracts (contract_id, contract_number, company_code, supplier, purchasing_org, purchasing_group, validity_start, validity_end, target_value, currency, release_status) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
            c.contract_id, c.contract_number, c.company_code, c.supplier, c.purchasing_org, c.purchasing_group, c.validity_start, c.validity_end, c.target_value, c.currency, c.release_status
        ).execute(&mut *tx).await?;

        for item in &c.items {
            sqlx::query!(
                "INSERT INTO contract_items (item_id, contract_id, item_number, material, short_text, target_quantity, unit, net_price, price_currency, plant) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                item.item_id, item.contract_id, item.item_number, item.material, item.short_text, item.target_quantity, item.unit, item.net_price, item.price_currency, item.plant
            ).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_number(&self, num: &str) -> Result<Option<Contract>> {
        let h = sqlx::query!("SELECT * FROM contracts WHERE contract_number = $1", num)
            .fetch_optional(&self.pool).await?;
        if let Some(h) = h {
            let items = sqlx::query!("SELECT * FROM contract_items WHERE contract_id = $1 ORDER BY item_number", h.contract_id)
                .fetch_all(&self.pool).await?;
            Ok(Some(Contract {
                contract_id: h.contract_id,
                contract_number: h.contract_number,
                company_code: h.company_code,
                supplier: h.supplier,
                purchasing_org: h.purchasing_org,
                purchasing_group: h.purchasing_group,
                validity_start: h.validity_start,
                validity_end: h.validity_end,
                target_value: h.target_value,
                currency: h.currency.unwrap_or_default(),
                release_status: h.release_status.unwrap_or_default(),
                created_at: h.created_at,
                items: items.into_iter().map(|i| ContractItem {
                    item_id: i.item_id,
                    contract_id: i.contract_id,
                    item_number: i.item_number,
                    material: i.material,
                    short_text: i.short_text,
                    target_quantity: i.target_quantity,
                    unit: i.unit.unwrap_or_default(),
                    net_price: i.net_price,
                    price_currency: i.price_currency.unwrap_or_default(),
                    plant: i.plant,
                }).collect(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn approve(&self, contract_id: uuid::Uuid, approved: bool) -> Result<()> {
        let status = if approved { "RELEASED" } else { "REJECTED" };
        sqlx::query!("UPDATE contracts SET release_status = $1 WHERE contract_id = $2", status, contract_id)
            .execute(&self.pool).await?;
        Ok(())
    }
}
