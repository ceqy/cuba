use sqlx::PgPool;
use crate::domain::{TransferOrder, TransferOrderItem};
use anyhow::Result;

pub struct TransferOrderRepository {
    pool: PgPool,
}

impl TransferOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, to: &TransferOrder) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO transfer_orders (to_id, to_number, warehouse_number, movement_type, reference_doc_type, reference_doc_number, status, created_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
            .bind(to.to_id)
            .bind(&to.to_number)
            .bind(&to.warehouse_number)
            .bind(&to.movement_type)
            .bind(&to.reference_doc_type)
            .bind(&to.reference_doc_number)
            .bind(&to.status)
            .bind(&to.created_by)
        .execute(&mut *tx).await?;

        for item in &to.items {
            sqlx::query(
                "INSERT INTO transfer_order_items (item_id, to_id, item_number, material, target_quantity, actual_quantity, unit, src_storage_type, src_storage_bin, dst_storage_type, dst_storage_bin, batch, confirmed) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)")
                .bind(item.item_id)
                .bind(item.to_id)
                .bind(item.item_number)
                .bind(&item.material)
                .bind(item.target_quantity)
                .bind(item.actual_quantity)
                .bind(&item.unit)
                .bind(&item.src_storage_type)
                .bind(&item.src_storage_bin)
                .bind(&item.dst_storage_type)
                .bind(&item.dst_storage_bin)
                .bind(&item.batch)
                .bind(item.confirmed)
            .execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_number(&self, wh: &str, to_num: &str) -> Result<Option<TransferOrder>> {
        let h = sqlx::query_as::<_, TransferOrder>("SELECT to_id, to_number, warehouse_number, movement_type, reference_doc_type, reference_doc_number, status, created_by, created_at FROM transfer_orders WHERE warehouse_number = $1 AND to_number = $2")
            .bind(wh)
            .bind(to_num)
            .fetch_optional(&self.pool).await?;
        if let Some(mut h) = h {
            let items = sqlx::query_as::<_, TransferOrderItem>("SELECT * FROM transfer_order_items WHERE to_id = $1 ORDER BY item_number")
                .bind(h.to_id)
                .fetch_all(&self.pool).await?;
            h.items = items;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }

    pub async fn confirm_order(&self, to_id: uuid::Uuid) -> Result<()> {
        sqlx::query("UPDATE transfer_orders SET status = 'CONFIRMED' WHERE to_id = $1")
            .bind(to_id)
            .execute(&self.pool).await?;
        sqlx::query("UPDATE transfer_order_items SET confirmed = true, actual_quantity = target_quantity WHERE to_id = $1")
            .bind(to_id)
            .execute(&self.pool).await?;
        Ok(())
    }
}
