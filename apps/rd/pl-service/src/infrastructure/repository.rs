use sqlx::PgPool;
use crate::domain::{BillOfMaterial, BOMItem};
use anyhow::Result;

pub struct BOMRepository {
    pool: PgPool,
}

impl BOMRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn sync_bom(&self, bom: &BillOfMaterial) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        
        // Upsert header
        sqlx::query!(
            r#"INSERT INTO bom_headers (bom_id, material, plant, bom_usage, bom_status, base_quantity, alternative_bom, valid_from)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               ON CONFLICT (material, plant, bom_usage, alternative_bom) DO UPDATE SET bom_status = $5, base_quantity = $6"#,
            bom.bom_id, bom.material, bom.plant, bom.bom_usage, bom.bom_status, bom.base_quantity, bom.alternative_bom, bom.valid_from
        ).execute(&mut *tx).await?;

        // Delete old items and insert new
        sqlx::query!("DELETE FROM bom_items WHERE bom_id = $1", bom.bom_id)
            .execute(&mut *tx).await?;

        for item in &bom.items {
            sqlx::query!(
                "INSERT INTO bom_items (item_id, bom_id, item_node, item_category, component_material, component_quantity, component_unit, item_text, recursive_allowed) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                item.item_id, item.bom_id, item.item_node, item.item_category, item.component_material, item.component_quantity, item.component_unit, item.item_text, item.recursive_allowed
            ).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_key(&self, material: &str, plant: &str, usage: &str) -> Result<Option<BillOfMaterial>> {
        let h = sqlx::query!("SELECT * FROM bom_headers WHERE material = $1 AND plant = $2 AND bom_usage = $3", material, plant, usage)
            .fetch_optional(&self.pool).await?;
        if let Some(h) = h {
            let items = sqlx::query!("SELECT * FROM bom_items WHERE bom_id = $1 ORDER BY item_node", h.bom_id)
                .fetch_all(&self.pool).await?;
            Ok(Some(BillOfMaterial {
                bom_id: h.bom_id,
                material: h.material,
                plant: h.plant,
                bom_usage: h.bom_usage.unwrap_or_else(|| "PRODUCTION".to_string()),
                bom_status: h.bom_status.unwrap_or_else(|| "ACTIVE".to_string()),
                base_quantity: h.base_quantity.unwrap_or_default(),
                alternative_bom: h.alternative_bom.unwrap_or_else(|| "1".to_string()),
                valid_from: h.valid_from,
                created_at: h.created_at,
                items: items.into_iter().map(|i| BOMItem {
                    item_id: i.item_id,
                    bom_id: i.bom_id,
                    item_node: i.item_node,
                    item_category: i.item_category.unwrap_or_else(|| "L".to_string()),
                    component_material: i.component_material,
                    component_quantity: i.component_quantity,
                    component_unit: i.component_unit.unwrap_or_else(|| "EA".to_string()),
                    item_text: i.item_text,
                    recursive_allowed: i.recursive_allowed.unwrap_or(false),
                }).collect(),
            }))
        } else {
            Ok(None)
        }
    }
}
