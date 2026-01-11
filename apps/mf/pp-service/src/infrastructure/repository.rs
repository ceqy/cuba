use sqlx::PgPool;
use crate::domain::PlannedOrder;
use anyhow::Result;

pub struct PlannedOrderRepository {
    pool: PgPool,
}

impl PlannedOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, order: &PlannedOrder) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO planned_orders (
                planned_order_id, planned_order_number, material, plant, planning_plant,
                order_quantity, quantity_unit, order_start_date, order_finish_date,
                mrp_controller, conversion_indicator, status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (planned_order_number) DO UPDATE SET
                updated_at = EXCLUDED.updated_at
            "#,
            order.planned_order_id, order.planned_order_number, order.material, order.plant,
            order.planning_plant, order.order_quantity, order.quantity_unit,
            order.order_start_date, order.order_finish_date,
            order.mrp_controller, order.conversion_indicator, order.status,
            order.created_at, order.updated_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_by_plant(&self, plant: &str) -> Result<Vec<PlannedOrder>> {
        let orders = sqlx::query_as!(
            PlannedOrder,
            r#"
            SELECT 
                planned_order_id, planned_order_number, material, plant, planning_plant,
                order_quantity, quantity_unit, order_start_date, order_finish_date,
                mrp_controller, 
                COALESCE(conversion_indicator, '') as conversion_indicator, 
                COALESCE(status, 'CREATED') as status, 
                created_at, updated_at
            FROM planned_orders
            WHERE plant = $1
            ORDER BY order_finish_date ASC
            LIMIT 100
            "#,
            plant
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(orders)
    }
}
