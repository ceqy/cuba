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

        sqlx::query(
            "INSERT INTO production_orders (order_id, order_number, order_type, material, plant, total_quantity, delivered_quantity, quantity_unit, basic_start_date, basic_finish_date, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)")
            .bind(order.order_id)
            .bind(&order.order_number)
            .bind(&order.order_type)
            .bind(&order.material)
            .bind(&order.plant)
            .bind(order.total_quantity)
            .bind(order.delivered_quantity)
            .bind(&order.quantity_unit)
            .bind(order.basic_start_date)
            .bind(order.basic_finish_date)
            .bind(&order.status)
            .bind(order.created_at)
            .bind(order.updated_at)
        .execute(&mut *tx).await?;

        for op in &order.operations {
            sqlx::query(
                "INSERT INTO production_operations (operation_id, order_id, operation_number, work_center, description, confirmed_yield, status) VALUES ($1, $2, $3, $4, $5, $6, $7)")
                .bind(op.operation_id)
                .bind(op.order_id)
                .bind(&op.operation_number)
                .bind(&op.work_center)
                .bind(&op.description)
                .bind(op.confirmed_yield)
                .bind(&op.status)
            .execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_number(&self, order_number: &str) -> Result<Option<ProductionOrder>> {
        let header = sqlx::query_as::<_, ProductionOrder>("SELECT order_id, order_number, order_type, material, plant, total_quantity, delivered_quantity, quantity_unit, basic_start_date, basic_finish_date, status, created_at, updated_at FROM production_orders WHERE order_number = $1")
            .bind(order_number)
            .fetch_optional(&self.pool).await?;

        if let Some(mut h) = header {
            let ops = sqlx::query_as::<_, ProductionOperation>("SELECT operation_id, order_id, operation_number, work_center, description, confirmed_yield, status FROM production_operations WHERE order_id = $1 ORDER BY operation_number")
                .bind(h.order_id)
                .fetch_all(&self.pool).await?;
            h.operations = ops;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }

    pub async fn save_confirmation(&self, conf: &ProductionConfirmation) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // 1. Save Confirmation
        sqlx::query(
            "INSERT INTO production_confirmations (confirmation_id, confirmation_number, order_id, operation_number, yield_quantity, scrap_quantity, final_confirmation, posting_date, personnel_number, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)")
            .bind(conf.confirmation_id)
            .bind(&conf.confirmation_number)
            .bind(conf.order_id)
            .bind(&conf.operation_number)
            .bind(conf.yield_quantity)
            .bind(conf.scrap_quantity)
            .bind(conf.final_confirmation)
            .bind(conf.posting_date)
            .bind(&conf.personnel_number)
            .bind(conf.created_at)
        .execute(&mut *tx).await?;

        // 2. Update Operation Status/Quantity
        // In a real system, this logic is complex. MVP: Just update confirmed yield.
         sqlx::query(
            "UPDATE production_operations SET confirmed_yield = confirmed_yield + $1, status = CASE WHEN $2 THEN 'CNF' ELSE 'PCNF' END WHERE order_id = $3 AND operation_number = $4")
            .bind(conf.yield_quantity)
            .bind(conf.final_confirmation)
            .bind(conf.order_id)
            .bind(&conf.operation_number)
        .execute(&mut *tx).await?;

        // 3. If Last Operation & Final Conf, Update Order Status (Simplified)
        if conf.final_confirmation {
             sqlx::query("UPDATE production_orders SET status = 'CNF' WHERE order_id = $1")
                .bind(conf.order_id)
            .execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }
}
