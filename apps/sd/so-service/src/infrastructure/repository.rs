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
        sqlx::query(
            "INSERT INTO sales_orders (order_id, order_number, order_type, sales_org, distribution_channel, division, sold_to_party, ship_to_party, customer_po, customer_po_date, document_date, requested_delivery_date, currency, net_value, pricing_procedure, shipping_conditions, overall_status, delivery_block, billing_block, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21) ON CONFLICT (order_number) DO UPDATE SET net_value = EXCLUDED.net_value, overall_status = EXCLUDED.overall_status, updated_at = EXCLUDED.updated_at")
            .bind(order.order_id)
            .bind(&order.order_number)
            .bind(&order.order_type)
            .bind(&order.sales_org)
            .bind(&order.distribution_channel)
            .bind(&order.division)
            .bind(&order.sold_to_party)
            .bind(&order.ship_to_party)
            .bind(&order.customer_po)
            .bind(order.customer_po_date)
            .bind(order.document_date)
            .bind(order.requested_delivery_date)
            .bind(&order.currency)
            .bind(order.net_value)
            .bind(&order.pricing_procedure)
            .bind(&order.shipping_conditions)
            .bind(&order.overall_status)
            .bind(&order.delivery_block)
            .bind(&order.billing_block)
            .bind(order.created_at)
            .bind(order.updated_at)
        .execute(&mut *tx).await?;

        // 2. Clear existing items
        sqlx::query("DELETE FROM sales_order_items WHERE order_id = $1")
            .bind(order.order_id)
        .execute(&mut *tx).await?;

        // 3. Insert Items and Schedule Lines
        for item in &order.items {
            sqlx::query(
                "INSERT INTO sales_order_items (item_id, order_id, item_number, material, item_description, order_quantity, sales_unit, plant, storage_location, net_value, tax_amount, item_category, rejection_reason, higher_level_item) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)")
                .bind(item.item_id)
                .bind(order.order_id)
                .bind(item.item_number)
                .bind(&item.material)
                .bind(&item.item_description)
                .bind(item.order_quantity)
                .bind(&item.sales_unit)
                .bind(&item.plant)
                .bind(&item.storage_location)
                .bind(item.net_value)
                .bind(item.tax_amount)
                .bind(&item.item_category)
                .bind(&item.rejection_reason)
                .bind(item.higher_level_item)
            .execute(&mut *tx).await?;

            for sl in &item.schedule_lines {
                sqlx::query(
                    "INSERT INTO sales_order_schedule_lines (schedule_line_id, item_id, schedule_line_number, delivery_date, confirmed_quantity) VALUES ($1, $2, $3, $4, $5)")
                    .bind(sl.schedule_line_id)
                    .bind(item.item_id)
                    .bind(sl.schedule_line_number)
                    .bind(sl.delivery_date)
                    .bind(sl.confirmed_quantity)
                .execute(&mut *tx).await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_number(&self, order_number: &str) -> Result<Option<SalesOrder>> {
        // Fetch Header
        let h = sqlx::query_as::<_, SalesOrder>("SELECT order_id, order_number, order_type, sales_org, distribution_channel, division, sold_to_party, ship_to_party, customer_po, customer_po_date, document_date, requested_delivery_date, currency, net_value, pricing_procedure, shipping_conditions, overall_status, delivery_block, billing_block, created_at, updated_at FROM sales_orders WHERE order_number = $1")
            .bind(order_number)
            .fetch_optional(&self.pool).await?;

        if let Some(mut h) = h {
            // Fetch Items
            let mut item_recs = sqlx::query_as::<_, SalesOrderItem>("SELECT item_id, order_id, item_number, material, item_description, order_quantity, sales_unit, plant, storage_location, net_value, tax_amount, item_category, rejection_reason, higher_level_item FROM sales_order_items WHERE order_id = $1 ORDER BY item_number ASC")
                .bind(h.order_id)
                .fetch_all(&self.pool).await?;

            for i in &mut item_recs {
                // Fetch Schedule Lines for Item
                let sl_recs = sqlx::query_as::<_, SalesOrderScheduleLine>("SELECT schedule_line_id, item_id, schedule_line_number, delivery_date, confirmed_quantity FROM sales_order_schedule_lines WHERE item_id = $1 ORDER BY schedule_line_number ASC")
                    .bind(i.item_id)
                    .fetch_all(&self.pool).await?;
                i.schedule_lines = sl_recs;
            }
            h.items = item_recs;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }
    
    pub async fn list(&self, sold_to_party: Option<String>, limit: i64) -> Result<Vec<SalesOrder>> {
        let orders = sqlx::query_as::<_, SalesOrder>("SELECT order_id, order_number, order_type, sales_org, distribution_channel, division, sold_to_party, ship_to_party, customer_po, customer_po_date, document_date, requested_delivery_date, currency, net_value, pricing_procedure, shipping_conditions, overall_status, delivery_block, billing_block, created_at, updated_at FROM sales_orders WHERE ($1::text IS NULL OR sold_to_party = $1) ORDER BY created_at DESC LIMIT $2")
            .bind(sold_to_party)
            .bind(limit)
            .fetch_all(&self.pool).await?;
        Ok(orders)
    }
}
