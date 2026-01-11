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
        sqlx::query!(
            "INSERT INTO transfer_orders (to_id, to_number, warehouse_number, movement_type, reference_doc_type, reference_doc_number, status, created_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            to.to_id, to.to_number, to.warehouse_number, to.movement_type, to.reference_doc_type, to.reference_doc_number, to.status, to.created_by
        ).execute(&mut *tx).await?;

        for item in &to.items {
            sqlx::query!(
                "INSERT INTO transfer_order_items (item_id, to_id, item_number, material, target_quantity, actual_quantity, unit, src_storage_type, src_storage_bin, dst_storage_type, dst_storage_bin, batch, confirmed) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
                item.item_id, item.to_id, item.item_number, item.material, item.target_quantity, item.actual_quantity, item.unit, item.src_storage_type, item.src_storage_bin, item.dst_storage_type, item.dst_storage_bin, item.batch, item.confirmed
            ).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_number(&self, wh: &str, to_num: &str) -> Result<Option<TransferOrder>> {
        let h = sqlx::query!("SELECT * FROM transfer_orders WHERE warehouse_number = $1 AND to_number = $2", wh, to_num)
            .fetch_optional(&self.pool).await?;
        if let Some(h) = h {
            let items = sqlx::query!("SELECT * FROM transfer_order_items WHERE to_id = $1 ORDER BY item_number", h.to_id)
                .fetch_all(&self.pool).await?;
            Ok(Some(TransferOrder {
                to_id: h.to_id,
                to_number: h.to_number,
                warehouse_number: h.warehouse_number,
                movement_type: h.movement_type,
                reference_doc_type: h.reference_doc_type,
                reference_doc_number: h.reference_doc_number,
                status: h.status.unwrap_or_else(|| "CREATED".to_string()),
                created_by: h.created_by,
                created_at: h.created_at,
                items: items.into_iter().map(|i| TransferOrderItem {
                    item_id: i.item_id,
                    to_id: i.to_id,
                    item_number: i.item_number,
                    material: i.material,
                    target_quantity: i.target_quantity,
                    actual_quantity: i.actual_quantity.unwrap_or_default(),
                    unit: i.unit.unwrap_or_else(|| "EA".to_string()),
                    src_storage_type: i.src_storage_type,
                    src_storage_bin: i.src_storage_bin,
                    dst_storage_type: i.dst_storage_type,
                    dst_storage_bin: i.dst_storage_bin,
                    batch: i.batch,
                    confirmed: i.confirmed.unwrap_or(false),
                }).collect(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn confirm_order(&self, to_id: uuid::Uuid) -> Result<()> {
        sqlx::query!("UPDATE transfer_orders SET status = 'CONFIRMED' WHERE to_id = $1", to_id)
            .execute(&self.pool).await?;
        sqlx::query!("UPDATE transfer_order_items SET confirmed = true, actual_quantity = target_quantity WHERE to_id = $1", to_id)
            .execute(&self.pool).await?;
        Ok(())
    }
}
