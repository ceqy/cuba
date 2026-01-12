use sqlx::{PgPool, Postgres, Transaction};
use crate::domain::{MaterialStock, MaterialDocument};
use anyhow::Result;
use rust_decimal::Decimal;

pub struct InventoryRepository {
    pool: PgPool,
}

impl InventoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_stock(
        &self,
        material: &str,
        plant: &str,
        storage_location: Option<&str>,
    ) -> Result<Vec<MaterialStock>> {
        let recs = sqlx::query_as::<_, MaterialStock>(
            r#"
            SELECT 
                stock_id, plant, storage_location, material, batch,
                unrestricted_quantity, quality_inspection_quantity, blocked_quantity,
                base_unit, last_movement_date, created_at, updated_at
            FROM material_stock
            WHERE material = $1 AND plant = $2
            AND ($3::text IS NULL OR storage_location = $3)
            "#)
            .bind(material)
            .bind(plant)
            .bind(storage_location)
        .fetch_all(&self.pool)
        .await?;

        Ok(recs)
    }

    pub async fn save_material_document(&self, doc: &MaterialDocument) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // 1. Insert Header
        sqlx::query(
            r#"
            INSERT INTO material_documents (
                document_id, document_number, fiscal_year, document_date, posting_date,
                document_type, reference_document, header_text, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#)
            .bind(doc.document_id)
            .bind(&doc.document_number)
            .bind(doc.fiscal_year)
            .bind(doc.document_date)
            .bind(doc.posting_date)
            .bind(&doc.document_type)
            .bind(&doc.reference_document)
            .bind(&doc.header_text)
            .bind(doc.created_at)
        .execute(&mut *tx)
        .await?;

        // 2. Process Items and Update Stock
        for item in &doc.items {
            sqlx::query(
                r#"
                INSERT INTO material_document_items (
                    item_id, document_id, line_item_number, movement_type, debit_credit_indicator,
                    material, plant, storage_location, batch, quantity, unit_of_measure, amount_lc
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                "#)
                .bind(item.item_id)
                .bind(item.document_id)
                .bind(item.line_item_number)
                .bind(&item.movement_type)
                .bind(&item.debit_credit_indicator)
                .bind(&item.material)
                .bind(&item.plant)
                .bind(&item.storage_location)
                .bind(item.batch.as_deref().unwrap_or(""))
                .bind(item.quantity)
                .bind(&item.unit_of_measure)
                .bind(item.amount_lc)
            .execute(&mut *tx)
            .await?;

            // Update Stock Logic
            self.update_stock(&mut tx, item).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn update_stock(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        item: &crate::domain::MaterialDocumentItem,
    ) -> Result<()> {
        let batch = item.batch.as_deref().unwrap_or("");
        
        // Simple UPSERT to initialize stock record if missing
        // Note: In real world, we might want to check if material exists in Plant (MARC) first.
        sqlx::query(
            r#"
            INSERT INTO material_stock (
                plant, storage_location, material, batch, base_unit
            ) VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (plant, storage_location, material, batch) DO NOTHING
            "#)
            .bind(&item.plant)
            .bind(&item.storage_location)
            .bind(&item.material)
            .bind(batch)
            .bind(&item.unit_of_measure)
        .execute(&mut **tx)
        .await?;
        
        // Determine sign based on Debit/Credit (S/H)
        // S = Debit (Stock Increase), H = Credit (Stock Decrease)
        let sign = if item.debit_credit_indicator == "S" { Decimal::ONE } else { Decimal::NEGATIVE_ONE };
        let delta = item.quantity * sign;

        // MVP: Only updating Unrestricted Use stock for now. 
        // Real-world would depend on Movement Type config (Quality, Blocked, etc.)
        sqlx::query(
            r#"
            UPDATE material_stock
            SET unrestricted_quantity = unrestricted_quantity + $1,
                last_movement_date = NOW(),
                updated_at = NOW()
            WHERE plant = $2 AND storage_location = $3 AND material = $4 AND batch = $5
            "#)
            .bind(delta)
            .bind(&item.plant)
            .bind(&item.storage_location)
            .bind(&item.material)
            .bind(batch)
        .execute(&mut **tx)
        .await?;

        // Check for negative stock if not allowed (omitted for MVP, assumed DB constraints or logic)
        
        Ok(())
    }
}
