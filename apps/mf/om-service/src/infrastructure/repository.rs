use crate::domain::{SubcontractingComponent, SubcontractingItem, SubcontractingOrder};
use anyhow::Result;
use sqlx::PgPool;

pub struct SubcontractingRepository {
    pool: PgPool,
}

impl SubcontractingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, order: &SubcontractingOrder) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO subcontracting_orders (order_id, purchase_order_number, supplier, company_code, purchasing_org, purchasing_group) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(order.order_id)
            .bind(&order.purchase_order_number)
            .bind(&order.supplier)
            .bind(&order.company_code)
            .bind(&order.purchasing_org)
            .bind(&order.purchasing_group)
        .execute(&mut *tx).await?;

        for item in &order.items {
            sqlx::query(
                "INSERT INTO subcontracting_items (item_id, order_id, item_number, finished_good_material, order_quantity, unit, plant) VALUES ($1, $2, $3, $4, $5, $6, $7)")
                .bind(item.item_id)
                .bind(item.order_id)
                .bind(item.item_number)
                .bind(&item.finished_good_material)
                .bind(item.order_quantity)
                .bind(&item.unit)
                .bind(&item.plant)
            .execute(&mut *tx).await?;

            for comp in &item.components {
                sqlx::query(
                    "INSERT INTO subcontracting_components (component_id, item_id, component_material, required_quantity, unit) VALUES ($1, $2, $3, $4, $5)")
                    .bind(comp.component_id)
                    .bind(comp.item_id)
                    .bind(&comp.component_material)
                    .bind(comp.required_quantity)
                    .bind(&comp.unit)
                .execute(&mut *tx).await?;
            }
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_po_number(&self, po_num: &str) -> Result<Option<SubcontractingOrder>> {
        let h = sqlx::query_as::<_, SubcontractingOrder>("SELECT order_id, purchase_order_number, supplier, company_code, purchasing_org, purchasing_group, created_at FROM subcontracting_orders WHERE purchase_order_number = $1")
            .bind(po_num)
            .fetch_optional(&self.pool).await?;
        if let Some(mut h) = h {
            let items = sqlx::query_as::<_, SubcontractingItem>("SELECT item_id, order_id, item_number, finished_good_material, order_quantity, received_quantity, unit, plant FROM subcontracting_items WHERE order_id = $1 ORDER BY item_number")
                .bind(h.order_id)
                .fetch_all(&self.pool).await?;
            let mut items_with_components = Vec::new();
            for mut item in items {
                let components = sqlx::query_as::<_, SubcontractingComponent>("SELECT component_id, item_id, component_material, required_quantity, issued_quantity, unit FROM subcontracting_components WHERE item_id = $1")
                    .bind(item.item_id)
                    .fetch_all(&self.pool).await?;
                item.components = components;
                items_with_components.push(item);
            }
            h.items = items_with_components;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }
}
