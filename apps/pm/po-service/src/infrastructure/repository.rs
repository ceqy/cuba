use crate::domain::{PurchaseOrder, PurchaseOrderItem, PurchaseOrderScheduleLine};
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PurchaseOrderRepository {
    pool: PgPool,
}

impl PurchaseOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, order: &PurchaseOrder) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // 1. Header
        sqlx::query(
            r#"
            INSERT INTO purchase_orders (
                order_id, order_number, document_type, company_code, purchasing_org, purchasing_group,
                supplier, order_date, currency, payment_terms, incoterms, incoterms_location,
                complete_delivery, release_status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            ON CONFLICT (order_number) DO UPDATE SET
                updated_at = EXCLUDED.updated_at
            "#)
            .bind(order.order_id)
            .bind(&order.order_number)
            .bind(order.document_type)
            .bind(&order.company_code)
            .bind(&order.purchasing_org)
            .bind(&order.purchasing_group)
            .bind(&order.supplier)
            .bind(order.order_date)
            .bind(&order.currency)
            .bind(&order.payment_terms)
            .bind(&order.incoterms)
            .bind(&order.incoterms_location)
            .bind(order.complete_delivery)
            .bind(order.release_status)
            .bind(order.created_at)
            .bind(order.updated_at)
        .execute(&mut *tx)
        .await?;

        // 2. Clear items (simplified)
        sqlx::query("DELETE FROM purchase_order_items WHERE order_id = $1")
            .bind(order.order_id)
            .execute(&mut *tx)
            .await?;

        // 3. Insert items
        for item in &order.items {
            sqlx::query(
                r#"
                INSERT INTO purchase_order_items (
                    item_id, order_id, item_number, item_category, material, short_text,
                    plant, storage_location, material_group, quantity, quantity_unit,
                    net_price, price_unit, currency, gr_based_iv, tax_code, 
                    account_assignment_category, requisition_number, requisition_item, deletion_indicator
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
                "#)
                .bind(item.item_id)
                .bind(order.order_id)
                .bind(item.item_number)
                .bind(item.item_category)
                .bind(&item.material)
                .bind(&item.short_text)
                .bind(&item.plant)
                .bind(&item.storage_location)
                .bind(&item.material_group)
                .bind(item.quantity)
                .bind(&item.quantity_unit)
                .bind(item.net_price)
                .bind(item.price_unit)
                .bind(&item.currency)
                .bind(item.gr_based_iv)
                .bind(&item.tax_code)
                .bind(&item.account_assignment_category)
                .bind(&item.requisition_number)
                .bind(item.requisition_item)
                .bind(item.deletion_indicator)
            .execute(&mut *tx)
            .await?;

            for sl in &item.schedule_lines {
                sqlx::query(
                    r#"
                    INSERT INTO purchase_order_schedule_lines (
                        schedule_line_id, item_id, schedule_line_number, delivery_date, scheduled_quantity, goods_receipt_quantity
                    )
                    VALUES ($1, $2, $3, $4, $5, $6)
                    "#)
                    .bind(sl.schedule_line_id)
                    .bind(item.item_id)
                    .bind(sl.schedule_line_number)
                    .bind(sl.delivery_date)
                    .bind(sl.scheduled_quantity)
                    .bind(sl.goods_receipt_quantity)
                 .execute(&mut *tx)
                 .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_number(&self, order_number: &str) -> Result<Option<PurchaseOrder>> {
        let h = sqlx::query_as::<_, PurchaseOrder>(
            r#"
            SELECT 
                order_id, order_number, document_type, company_code, purchasing_org, purchasing_group,
                supplier, order_date, currency, payment_terms, incoterms, incoterms_location,
                complete_delivery, release_status, created_at, updated_at
            FROM purchase_orders 
            WHERE order_number = $1
            "#)
            .bind(order_number)
        .fetch_optional(&self.pool).await?;

        if let Some(mut h) = h {
            let items_recs = sqlx::query_as::<_, PurchaseOrderItem>(
                r#"SELECT * FROM purchase_order_items WHERE order_id = $1 ORDER BY item_number ASC"#)
                .bind(h.order_id)
            .fetch_all(&self.pool).await?;

            let mut items = Vec::new();
            for mut i in items_recs {
                let sl_recs = sqlx::query_as::<_, PurchaseOrderScheduleLine>(
                    r#"SELECT 
                         schedule_line_id, item_id, schedule_line_number, delivery_date, scheduled_quantity, goods_receipt_quantity
                       FROM purchase_order_schedule_lines WHERE item_id = $1 ORDER BY schedule_line_number ASC"#)
                    .bind(i.item_id)
                 .fetch_all(&self.pool).await?;

                i.schedule_lines = sl_recs;
                items.push(i);
            }
            h.items = items;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }
}
