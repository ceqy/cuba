use crate::domain::{InspectionCharacteristic, InspectionLot};
use anyhow::Result;
use sqlx::PgPool;

pub struct InspectionLotRepository {
    pool: PgPool,
}

impl InspectionLotRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_lot(&self, lot: &InspectionLot) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            "INSERT INTO inspection_lots (lot_id, inspection_lot_number, material, plant, lot_quantity, quantity_unit, origin, creation_date, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)")
            .bind(lot.lot_id)
            .bind(&lot.inspection_lot_number)
            .bind(&lot.material)
            .bind(&lot.plant)
            .bind(lot.lot_quantity)
            .bind(&lot.quantity_unit)
            .bind(&lot.origin)
            .bind(lot.creation_date)
            .bind(lot.created_at)
            .bind(lot.updated_at)
        .execute(&mut *tx).await?;

        for char in &lot.characteristics {
            sqlx::query(
                "INSERT INTO inspection_characteristics (char_id, lot_id, characteristic_number, description, inspection_method, result_status) VALUES ($1, $2, $3, $4, $5, $6)")
                .bind(char.char_id)
                .bind(char.lot_id)
                .bind(&char.characteristic_number)
                .bind(&char.description)
                .bind(&char.inspection_method)
                .bind(&char.result_status)
            .execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_number(&self, lot_number: &str) -> Result<Option<InspectionLot>> {
        let header = sqlx::query_as::<_, InspectionLot>("SELECT lot_id, inspection_lot_number, material, plant, lot_quantity, quantity_unit, origin, creation_date, ud_code, ud_date, ud_note, created_at, updated_at FROM inspection_lots WHERE inspection_lot_number = $1")
            .bind(lot_number)
            .fetch_optional(&self.pool).await?;

        if let Some(mut h) = header {
            let chars = sqlx::query_as::<_, InspectionCharacteristic>("SELECT char_id, lot_id, characteristic_number, description, inspection_method, result_value, result_status FROM inspection_characteristics WHERE lot_id = $1 ORDER BY characteristic_number")
                .bind(h.lot_id)
                .fetch_all(&self.pool).await?;
            h.characteristics = chars;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }

    pub async fn update_result(
        &self,
        lot_id: uuid::Uuid,
        char_num: &str,
        value: &str,
    ) -> Result<()> {
        sqlx::query("UPDATE inspection_characteristics SET result_value = $1, result_status = '5' WHERE lot_id = $2 AND characteristic_number = $3")
            .bind(value)
            .bind(lot_id)
            .bind(char_num)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn make_usage_decision(
        &self,
        lot_id: uuid::Uuid,
        ud_code: &str,
        note: &str,
    ) -> Result<()> {
        sqlx::query("UPDATE inspection_lots SET ud_code = $1, ud_note = $2, ud_date = NOW(), updated_at = NOW() WHERE lot_id = $3")
            .bind(ud_code)
            .bind(note)
            .bind(lot_id)
        .execute(&self.pool).await?;
        Ok(())
    }
}
