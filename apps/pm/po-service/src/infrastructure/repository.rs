use sqlx::PgPool;
use crate::domain::{PurchaseOrder, PurchaseOrderItem, PurchaseOrderScheduleLine};
use anyhow::Result;
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
        sqlx::query!(
            r#"
            INSERT INTO purchase_orders (
                order_id, order_number, document_type, company_code, purchasing_org, purchasing_group,
                supplier, order_date, currency, payment_terms, incoterms, incoterms_location,
                complete_delivery, release_status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            ON CONFLICT (order_number) DO UPDATE SET
                updated_at = EXCLUDED.updated_at
            "#,
            order.order_id, order.order_number, order.document_type, order.company_code,
            order.purchasing_org, order.purchasing_group, order.supplier, order.order_date,
            order.currency, order.payment_terms, order.incoterms, order.incoterms_location,
            order.complete_delivery, order.release_status, order.created_at, order.updated_at
        )
        .execute(&mut *tx)
        .await?;

        // 2. Clear items (simplified)
        sqlx::query!("DELETE FROM purchase_order_items WHERE order_id = $1", order.order_id)
            .execute(&mut *tx)
            .await?;
            
        // 3. Insert items
        for item in &order.items {
            sqlx::query!(
                r#"
                INSERT INTO purchase_order_items (
                    item_id, order_id, item_number, item_category, material, short_text,
                    plant, storage_location, material_group, quantity, quantity_unit,
                    net_price, price_unit, currency, gr_based_iv, tax_code, 
                    account_assignment_category, requisition_number, requisition_item, deletion_indicator
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
                "#,
                item.item_id, order.order_id, item.item_number, item.item_category, item.material, item.short_text,
                item.plant, item.storage_location, item.material_group, item.quantity, item.quantity_unit,
                item.net_price, item.price_unit, item.currency, item.gr_based_iv, item.tax_code,
                item.account_assignment_category, item.requisition_number, item.requisition_item, item.deletion_indicator
            )
            .execute(&mut *tx)
            .await?;
            
            for sl in &item.schedule_lines {
                 sqlx::query!(
                    r#"
                    INSERT INTO purchase_order_schedule_lines (
                        schedule_line_id, item_id, schedule_line_number, delivery_date, scheduled_quantity, goods_receipt_quantity
                    )
                    VALUES ($1, $2, $3, $4, $5, $6)
                    "#,
                    sl.schedule_line_id, item.item_id, sl.schedule_line_number, sl.delivery_date, sl.scheduled_quantity, sl.goods_receipt_quantity
                 )
                 .execute(&mut *tx)
                 .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_number(&self, order_number: &str) -> Result<Option<PurchaseOrder>> {
        let rec = sqlx::query!(
            r#"
            SELECT 
                order_id, order_number, document_type, company_code, purchasing_org, purchasing_group,
                supplier, order_date, currency, payment_terms, incoterms, incoterms_location,
                complete_delivery, release_status, created_at, updated_at
            FROM purchase_orders 
            WHERE order_number = $1
            "#,
            order_number
        ).fetch_optional(&self.pool).await?;

        if let Some(h) = rec {
            let items_recs = sqlx::query!(
                r#"SELECT * FROM purchase_order_items WHERE order_id = $1 ORDER BY item_number ASC"#,
                h.order_id
            ).fetch_all(&self.pool).await?;
            
            let mut items = Vec::new();
            for i in items_recs {
                 let sl_recs = sqlx::query_as!(
                    PurchaseOrderScheduleLine,
                    r#"SELECT 
                         schedule_line_id, item_id, schedule_line_number, delivery_date, scheduled_quantity, goods_receipt_quantity
                       FROM purchase_order_schedule_lines WHERE item_id = $1 ORDER BY schedule_line_number ASC"#,
                    i.item_id
                 ).fetch_all(&self.pool).await?;
                 
                 items.push(PurchaseOrderItem {
                     item_id: i.item_id,
                     order_id: i.order_id,
                     item_number: i.item_number,
                     item_category: i.item_category,
                     material: i.material,
                     short_text: i.short_text,
                     plant: i.plant,
                     storage_location: i.storage_location,
                     material_group: i.material_group,
                     quantity: i.quantity,
                     quantity_unit: i.quantity_unit,
                     net_price: i.net_price,
                     price_unit: i.price_unit.unwrap_or(1),
                     currency: i.currency,
                     gr_based_iv: i.gr_based_iv.unwrap_or(true),
                     tax_code: i.tax_code,
                     account_assignment_category: i.account_assignment_category,
                     requisition_number: i.requisition_number,
                     requisition_item: i.requisition_item,
                     deletion_indicator: i.deletion_indicator.unwrap_or(false),
                     schedule_lines: sl_recs,
                 });
            }

            Ok(Some(PurchaseOrder {
                order_id: h.order_id,
                order_number: h.order_number,
                document_type: h.document_type,
                company_code: h.company_code,
                purchasing_org: h.purchasing_org,
                purchasing_group: h.purchasing_group,
                supplier: h.supplier,
                order_date: h.order_date,
                currency: h.currency,
                payment_terms: h.payment_terms,
                incoterms: h.incoterms,
                incoterms_location: h.incoterms_location,
                complete_delivery: h.complete_delivery.unwrap_or(false),
                release_status: h.release_status,
                items,
                created_at: h.created_at,
                updated_at: h.updated_at,
            }))
        } else {
            Ok(None)
        }
    }
}
