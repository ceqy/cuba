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
        sqlx::query(
            "INSERT INTO bom_headers (bom_id, material, plant, bom_usage, bom_status, base_quantity, alternative_bom, valid_from) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT (material, plant, bom_usage, alternative_bom) DO UPDATE SET bom_status = EXCLUDED.bom_status, base_quantity = EXCLUDED.base_quantity")
            .bind(bom.bom_id)
            .bind(&bom.material)
            .bind(&bom.plant)
            .bind(&bom.bom_usage)
            .bind(&bom.bom_status)
            .bind(bom.base_quantity)
            .bind(&bom.alternative_bom)
            .bind(bom.valid_from)
        .execute(&mut *tx).await?;

        // Delete old items
        sqlx::query("DELETE FROM bom_items WHERE bom_id = $1")
            .bind(bom.bom_id)
        .execute(&mut *tx).await?;

        for item in &bom.items {
            sqlx::query(
                "INSERT INTO bom_items (item_id, bom_id, item_node, item_category, component_material, component_quantity, component_unit, item_text, recursive_allowed) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
                .bind(item.item_id)
                .bind(item.bom_id)
                .bind(&item.item_node)
                .bind(&item.item_category)
                .bind(&item.component_material)
                .bind(item.component_quantity)
                .bind(&item.component_unit)
                .bind(&item.item_text)
                .bind(item.recursive_allowed)
            .execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_key(&self, material: &str, plant: &str, usage: &str) -> Result<Option<BillOfMaterial>> {
        let h = sqlx::query_as::<_, BillOfMaterial>("SELECT bom_id, material, plant, bom_usage, bom_status, base_quantity, alternative_bom, valid_from, created_at FROM bom_headers WHERE material = $1 AND plant = $2 AND bom_usage = $3")
            .bind(material)
            .bind(plant)
            .bind(usage)
            .fetch_optional(&self.pool).await?;
        if let Some(mut h) = h {
            let items = sqlx::query_as::<_, BOMItem>("SELECT item_id, bom_id, item_node, item_category, component_material, component_quantity, component_unit, item_text, recursive_allowed FROM bom_items WHERE bom_id = $1 ORDER BY item_node")
                .bind(h.bom_id)
                .fetch_all(&self.pool).await?;
            h.items = items;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }
}
