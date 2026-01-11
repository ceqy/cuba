use sqlx::PgPool;
use crate::domain::{ProductionOrder, ProductionOperation, ProductionConfirmation};
use anyhow::Result;

pub struct ProductionOrderRepository {
    pool: PgPool,
}

impl ProductionOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_order(&self, order: &ProductionOrder) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            r#"
            INSERT INTO production_orders (
                order_id, order_number, order_type, material, plant,
                total_quantity, delivered_quantity, quantity_unit,
                basic_start_date, basic_finish_date, status, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
            order.order_id, order.order_number, order.order_type,
            order.material, order.plant,
            order.total_quantity, order.delivered_quantity, order.quantity_unit,
            order.basic_start_date, order.basic_finish_date,
            order.status, order.created_at, order.updated_at
        ).execute(&mut *tx).await?;

        for op in &order.operations {
            sqlx::query!(
                r#"
                INSERT INTO production_operations (
                    operation_id, order_id, operation_number, work_center, description,
                    confirmed_yield, status
                ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                op.operation_id, op.order_id, op.operation_number,
                op.work_center, op.description, op.confirmed_yield, op.status
            ).execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_number(&self, order_number: &str) -> Result<Option<ProductionOrder>> {
        let header = sqlx::query!(
            r#"
            SELECT 
                order_id, order_number, order_type, material, plant,
                total_quantity, delivered_quantity, quantity_unit,
                basic_start_date, basic_finish_date, status, created_at, updated_at
            FROM production_orders
            WHERE order_number = $1
            "#,
            order_number
        ).fetch_optional(&self.pool).await?;

        if let Some(h) = header {
            let ops = sqlx::query!(
                r#"
                SELECT 
                    operation_id, order_id, operation_number, work_center, 
                    description, confirmed_yield, status
                FROM production_operations 
                WHERE order_id = $1 
                ORDER BY operation_number
                "#,
                h.order_id
            ).fetch_all(&self.pool).await?;

            let operations = ops.into_iter().map(|op| ProductionOperation {
                operation_id: op.operation_id,
                order_id: op.order_id,
                operation_number: op.operation_number,
                work_center: op.work_center,
                description: op.description,
                confirmed_yield: op.confirmed_yield.unwrap_or_default(),
                status: op.status.unwrap_or_else(|| "CREATED".to_string()),
            }).collect();

            Ok(Some(ProductionOrder {
                order_id: h.order_id,
                order_number: h.order_number,
                order_type: h.order_type,
                material: h.material,
                plant: h.plant,
                total_quantity: h.total_quantity,
                delivered_quantity: h.delivered_quantity.unwrap_or_default(),
                quantity_unit: h.quantity_unit,
                basic_start_date: h.basic_start_date,
                basic_finish_date: h.basic_finish_date,
                status: h.status.unwrap_or_else(|| "CREATED".to_string()),
                created_at: h.created_at,
                updated_at: h.updated_at,
                operations,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn save_confirmation(&self, conf: &ProductionConfirmation) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // 1. Save Confirmation
        sqlx::query!(
            r#"
            INSERT INTO production_confirmations (
                confirmation_id, confirmation_number, order_id, operation_number,
                yield_quantity, scrap_quantity, final_confirmation,
                posting_date, personnel_number, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            conf.confirmation_id, conf.confirmation_number, conf.order_id,
            conf.operation_number, conf.yield_quantity, conf.scrap_quantity,
            conf.final_confirmation, conf.posting_date, conf.personnel_number,
            conf.created_at
        ).execute(&mut *tx).await?;

        // 2. Update Operation Status/Quantity
        // In a real system, this logic is complex. MVP: Just update confirmed yield.
         sqlx::query!(
            r#"
            UPDATE production_operations 
            SET confirmed_yield = confirmed_yield + $1,
                status = CASE WHEN $2 THEN 'CNF' ELSE 'PCNF' END
            WHERE order_id = $3 AND operation_number = $4
            "#,
            conf.yield_quantity, // Add to yield
            conf.final_confirmation,
            conf.order_id,
            conf.operation_number
        ).execute(&mut *tx).await?;

        // 3. If Last Operation & Final Conf, Update Order Status (Simplified)
        if conf.final_confirmation {
             sqlx::query!(
                r#"
                UPDATE production_orders
                SET status = 'CNF' -- Confirmed
                WHERE order_id = $1
                "#,
                conf.order_id
            ).execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }
}
