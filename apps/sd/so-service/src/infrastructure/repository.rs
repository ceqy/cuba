use sqlx::PgPool;
use crate::domain::{SalesOrder, SalesOrderItem, SalesOrderScheduleLine};
use anyhow::Result;
use uuid::Uuid;
use chrono::NaiveDate;
use rust_decimal::Decimal;

pub struct SalesOrderRepository {
    pool: PgPool,
}

impl SalesOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, order: &SalesOrder) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // 1. Insert Header
        sqlx::query!(
            r#"
            INSERT INTO sales_orders (
                order_id, order_number, order_type, sales_org, distribution_channel, division,
                sold_to_party, ship_to_party, customer_po, customer_po_date,
                document_date, requested_delivery_date, currency, net_value,
                pricing_procedure, shipping_conditions, overall_status,
                delivery_block, billing_block, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            ON CONFLICT (order_number) DO UPDATE SET
                net_value = EXCLUDED.net_value,
                overall_status = EXCLUDED.overall_status,
                updated_at = EXCLUDED.updated_at
            "#,
            order.order_id, order.order_number, order.order_type, order.sales_org, order.distribution_channel, order.division,
            order.sold_to_party, order.ship_to_party, order.customer_po, order.customer_po_date,
            order.document_date, order.requested_delivery_date, order.currency, order.net_value,
            order.pricing_procedure, order.shipping_conditions, order.overall_status,
            order.delivery_block, order.billing_block, order.created_at, order.updated_at
        )
        .execute(&mut *tx)
        .await?;

        // 2. Clear existing items (simple replacement strategy for MVP)
        // Ideally we would diff, but full replacement is safer for consistency here
        sqlx::query!("DELETE FROM sales_order_items WHERE order_id = $1", order.order_id)
            .execute(&mut *tx)
            .await?;

        // 3. Insert Items and Schedule Lines
        for item in &order.items {
            sqlx::query!(
                r#"
                INSERT INTO sales_order_items (
                    item_id, order_id, item_number, material, item_description,
                    order_quantity, sales_unit, plant, storage_location,
                    net_value, tax_amount, item_category, rejection_reason, higher_level_item
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                "#,
                item.item_id, order.order_id, item.item_number, item.material, item.item_description,
                item.order_quantity, item.sales_unit, item.plant, item.storage_location,
                item.net_value, item.tax_amount, item.item_category, item.rejection_reason, item.higher_level_item
            )
            .execute(&mut *tx)
            .await?;

            for sl in &item.schedule_lines {
                sqlx::query!(
                    r#"
                    INSERT INTO sales_order_schedule_lines (
                        schedule_line_id, item_id, schedule_line_number, delivery_date, confirmed_quantity
                    )
                    VALUES ($1, $2, $3, $4, $5)
                    "#,
                    sl.schedule_line_id, item.item_id, sl.schedule_line_number, sl.delivery_date, sl.confirmed_quantity
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_number(&self, order_number: &str) -> Result<Option<SalesOrder>> {
        // Fetch Header
        let header_rec = sqlx::query!(
            r#"SELECT * FROM sales_orders WHERE order_number = $1"#,
            order_number
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(h) = header_rec {
            // Fetch Items
            let item_recs = sqlx::query!(
                r#"SELECT * FROM sales_order_items WHERE order_id = $1 ORDER BY item_number ASC"#,
                h.order_id
            )
            .fetch_all(&self.pool)
            .await?;

            let mut items = Vec::new();
            for i in item_recs {
                // Fetch Schedule Lines for Item
                let sl_recs = sqlx::query!(
                    r#"SELECT * FROM sales_order_schedule_lines WHERE item_id = $1 ORDER BY schedule_line_number ASC"#,
                    i.item_id
                )
                .fetch_all(&self.pool)
                .await?;

                let schedule_lines = sl_recs.into_iter().map(|sl| SalesOrderScheduleLine {
                    schedule_line_id: sl.schedule_line_id,
                    item_id: sl.item_id,
                    schedule_line_number: sl.schedule_line_number,
                    delivery_date: sl.delivery_date,
                    confirmed_quantity: sl.confirmed_quantity,
                }).collect();

                items.push(SalesOrderItem {
                    item_id: i.item_id,
                    order_id: i.order_id,
                    item_number: i.item_number,
                    material: i.material,
                    item_description: i.item_description,
                    order_quantity: i.order_quantity,
                    sales_unit: i.sales_unit,
                    plant: i.plant,
                    storage_location: i.storage_location,
                    net_value: i.net_value,
                    tax_amount: i.tax_amount,
                    item_category: i.item_category,
                    rejection_reason: i.rejection_reason,
                    higher_level_item: i.higher_level_item,
                    schedule_lines,
                });
            }

            Ok(Some(SalesOrder {
                order_id: h.order_id,
                order_number: h.order_number,
                order_type: h.order_type,
                sales_org: h.sales_org,
                distribution_channel: h.distribution_channel,
                division: h.division,
                sold_to_party: h.sold_to_party,
                ship_to_party: h.ship_to_party,
                customer_po: h.customer_po,
                customer_po_date: h.customer_po_date,
                document_date: h.document_date,
                requested_delivery_date: h.requested_delivery_date,
                currency: h.currency,
                net_value: h.net_value,
                pricing_procedure: h.pricing_procedure,
                shipping_conditions: h.shipping_conditions,
                overall_status: h.overall_status,
                delivery_block: h.delivery_block,
                billing_block: h.billing_block,
                items,
                created_at: h.created_at,
                updated_at: h.updated_at,
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn list(&self, sold_to_party: Option<String>, limit: i64) -> Result<Vec<SalesOrder>> {
        // Simplified list query (just headers for MVP)
        // In reality we would use dynamic query filtering
         let recs = sqlx::query!(
            r#"
            SELECT * FROM sales_orders 
            WHERE ($1::text IS NULL OR sold_to_party = $1)
            ORDER BY created_at DESC 
            LIMIT $2
            "#,
            sold_to_party,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        // Mapping (omitting items for summary list)
        let orders = recs.into_iter().map(|h| SalesOrder {
                order_id: h.order_id,
                order_number: h.order_number,
                order_type: h.order_type,
                sales_org: h.sales_org,
                distribution_channel: h.distribution_channel,
                division: h.division,
                sold_to_party: h.sold_to_party,
                ship_to_party: h.ship_to_party,
                customer_po: h.customer_po,
                customer_po_date: h.customer_po_date,
                document_date: h.document_date,
                requested_delivery_date: h.requested_delivery_date,
                currency: h.currency,
                net_value: h.net_value,
                pricing_procedure: h.pricing_procedure,
                shipping_conditions: h.shipping_conditions,
                overall_status: h.overall_status,
                delivery_block: h.delivery_block,
                billing_block: h.billing_block,
                items: vec![], // Empty items for list view
                created_at: h.created_at,
                updated_at: h.updated_at,
        }).collect();
        
        Ok(orders)
    }
}
