use sqlx::PgPool;
use crate::domain::{SubcontractingOrder, SubcontractingItem, SubcontractingComponent};
use anyhow::Result;

pub struct SubcontractingRepository {
    pool: PgPool,
}

impl SubcontractingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, order: &SubcontractingOrder) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            "INSERT INTO subcontracting_orders (order_id, purchase_order_number, supplier, company_code, purchasing_org, purchasing_group) VALUES ($1, $2, $3, $4, $5, $6)",
            order.order_id, order.purchase_order_number, order.supplier, order.company_code, order.purchasing_org, order.purchasing_group
        ).execute(&mut *tx).await?;

        for item in &order.items {
            sqlx::query!(
                "INSERT INTO subcontracting_items (item_id, order_id, item_number, finished_good_material, order_quantity, unit, plant) VALUES ($1, $2, $3, $4, $5, $6, $7)",
                item.item_id, item.order_id, item.item_number, item.finished_good_material, item.order_quantity, item.unit, item.plant
            ).execute(&mut *tx).await?;

            for comp in &item.components {
                sqlx::query!(
                    "INSERT INTO subcontracting_components (component_id, item_id, component_material, required_quantity, unit) VALUES ($1, $2, $3, $4, $5)",
                    comp.component_id, comp.item_id, comp.component_material, comp.required_quantity, comp.unit
                ).execute(&mut *tx).await?;
            }
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_po_number(&self, po_num: &str) -> Result<Option<SubcontractingOrder>> {
        let h = sqlx::query!("SELECT * FROM subcontracting_orders WHERE purchase_order_number = $1", po_num)
            .fetch_optional(&self.pool).await?;
        if let Some(h) = h {
            let items = sqlx::query!("SELECT * FROM subcontracting_items WHERE order_id = $1 ORDER BY item_number", h.order_id)
                .fetch_all(&self.pool).await?;
            let mut items_with_components = Vec::new();
            for item in items {
                let components = sqlx::query!("SELECT * FROM subcontracting_components WHERE item_id = $1", item.item_id)
                    .fetch_all(&self.pool).await?;
                items_with_components.push(SubcontractingItem {
                    item_id: item.item_id,
                    order_id: item.order_id,
                    item_number: item.item_number,
                    finished_good_material: item.finished_good_material,
                    order_quantity: item.order_quantity,
                    received_quantity: item.received_quantity.unwrap_or_default(),
                    unit: item.unit.unwrap_or_default(),
                    plant: item.plant,
                    components: components.into_iter().map(|c| SubcontractingComponent {
                        component_id: c.component_id,
                        item_id: c.item_id,
                        component_material: c.component_material,
                        required_quantity: c.required_quantity,
                        issued_quantity: c.issued_quantity.unwrap_or_default(),
                        unit: c.unit.unwrap_or_default(),
                    }).collect(),
                });
            }
            Ok(Some(SubcontractingOrder {
                order_id: h.order_id,
                purchase_order_number: h.purchase_order_number,
                supplier: h.supplier,
                company_code: h.company_code,
                purchasing_org: h.purchasing_org,
                purchasing_group: h.purchasing_group,
                created_at: h.created_at,
                items: items_with_components,
            }))
        } else {
            Ok(None)
        }
    }
}
