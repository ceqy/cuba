use sqlx::PgPool;
use crate::domain::{InspectionLot, InspectionCharacteristic};
use anyhow::Result;

pub struct InspectionLotRepository {
    pool: PgPool,
}

impl InspectionLotRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_lot(&self, lot: &InspectionLot) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            r#"
            INSERT INTO inspection_lots (
                lot_id, inspection_lot_number, material, plant,
                lot_quantity, quantity_unit, origin, creation_date,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            lot.lot_id, lot.inspection_lot_number, lot.material, lot.plant,
            lot.lot_quantity, lot.quantity_unit, lot.origin, lot.creation_date,
            lot.created_at, lot.updated_at
        ).execute(&mut *tx).await?;

        for char in &lot.characteristics {
            sqlx::query!(
                r#"
                INSERT INTO inspection_characteristics (
                    char_id, lot_id, characteristic_number, description, inspection_method,
                    result_status
                ) VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                char.char_id, char.lot_id, char.characteristic_number,
                char.description, char.inspection_method, char.result_status
            ).execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_number(&self, lot_number: &str) -> Result<Option<InspectionLot>> {
        let header = sqlx::query!(
            r#"
            SELECT 
                lot_id, inspection_lot_number, material, plant,
                lot_quantity, quantity_unit, origin, creation_date,
                ud_code, ud_date, ud_note, created_at, updated_at
            FROM inspection_lots
            WHERE inspection_lot_number = $1
            "#,
            lot_number
        ).fetch_optional(&self.pool).await?;

        if let Some(h) = header {
            let chars = sqlx::query!(
                r#"
                SELECT 
                     char_id, lot_id, characteristic_number, description,
                     inspection_method, result_value, result_status
                FROM inspection_characteristics WHERE lot_id = $1 
                ORDER BY characteristic_number
                "#,
                h.lot_id
            ).fetch_all(&self.pool).await?;

            let characteristics = chars.into_iter().map(|c| InspectionCharacteristic {
                char_id: c.char_id,
                lot_id: c.lot_id,
                characteristic_number: c.characteristic_number,
                description: c.description,
                inspection_method: c.inspection_method,
                result_value: c.result_value,
                result_status: c.result_status.unwrap_or_else(|| "0".to_string()),
            }).collect();

            Ok(Some(InspectionLot {
                lot_id: h.lot_id,
                inspection_lot_number: h.inspection_lot_number,
                material: h.material,
                plant: h.plant,
                lot_quantity: h.lot_quantity,
                quantity_unit: h.quantity_unit,
                origin: h.origin,
                creation_date: h.creation_date,
                ud_code: h.ud_code,
                ud_date: h.ud_date,
                ud_note: h.ud_note,
                created_at: h.created_at,
                updated_at: h.updated_at,
                characteristics,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_result(&self, lot_id: uuid::Uuid, char_num: &str, value: &str) -> Result<()> {
         sqlx::query!(
            r#"
            UPDATE inspection_characteristics
            SET result_value = $1, result_status = '5'
            WHERE lot_id = $2 AND characteristic_number = $3
            "#,
            value,
            lot_id,
            char_num
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn make_usage_decision(&self, lot_id: uuid::Uuid, ud_code: &str, note: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE inspection_lots
            SET ud_code = $1, ud_note = $2, ud_date = NOW(), updated_at = NOW()
            WHERE lot_id = $3
            "#,
            ud_code,
            note,
            lot_id
        ).execute(&self.pool).await?;
        Ok(())
    }
}
