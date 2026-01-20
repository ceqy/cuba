use crate::domain::{MaintenanceNotification, MaintenanceOperation, MaintenanceOrder};
use anyhow::Result;
use sqlx::PgPool;

pub struct MaintenanceRepository {
    pool: PgPool,
}

impl MaintenanceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_notification(&self, n: &MaintenanceNotification) -> Result<()> {
        sqlx::query(
            "INSERT INTO maintenance_notifications (notification_id, notification_number, notification_type, description, equipment_number, functional_location, reported_by, reported_date, priority, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)")
            .bind(n.notification_id)
            .bind(&n.notification_number)
            .bind(&n.notification_type)
            .bind(&n.description)
            .bind(&n.equipment_number)
            .bind(&n.functional_location)
            .bind(&n.reported_by)
            .bind(n.reported_date)
            .bind(&n.priority)
            .bind(&n.status)
            .bind(n.created_at)
            .bind(n.updated_at)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn create_order(&self, o: &MaintenanceOrder) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO maintenance_orders (order_id, order_number, order_type, description, notification_number, equipment_number, functional_location, maintenance_plant, planning_plant, main_work_center, system_status, priority, basic_start_date, basic_finish_date, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)")
            .bind(o.order_id)
            .bind(&o.order_number)
            .bind(&o.order_type)
            .bind(&o.description)
            .bind(&o.notification_number)
            .bind(&o.equipment_number)
            .bind(&o.functional_location)
            .bind(&o.maintenance_plant)
            .bind(&o.planning_plant)
            .bind(&o.main_work_center)
            .bind(&o.system_status)
            .bind(&o.priority)
            .bind(o.basic_start_date)
            .bind(o.basic_finish_date)
            .bind(o.created_at)
            .bind(o.updated_at)
        .execute(&mut *tx).await?;

        for op in &o.operations {
            sqlx::query(
                "INSERT INTO maintenance_operations (operation_id, order_id, operation_number, description, work_center, planned_work_duration, actual_work_duration, work_unit, status) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
                .bind(op.operation_id)
                .bind(op.order_id)
                .bind(&op.operation_number)
                .bind(&op.description)
                .bind(&op.work_center)
                .bind(op.planned_work_duration)
                .bind(op.actual_work_duration)
                .bind(&op.work_unit)
                .bind(&op.status)
            .execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_order_by_number(
        &self,
        order_number: &str,
    ) -> Result<Option<MaintenanceOrder>> {
        let h = sqlx::query_as::<_, MaintenanceOrder>("SELECT order_id, order_number, order_type, description, notification_number, equipment_number, functional_location, maintenance_plant, planning_plant, main_work_center, system_status, priority, basic_start_date, basic_finish_date, created_at, updated_at FROM maintenance_orders WHERE order_number = $1")
            .bind(order_number)
            .fetch_optional(&self.pool).await?;

        if let Some(mut h) = h {
            let ops = sqlx::query_as::<_, MaintenanceOperation>("SELECT * FROM maintenance_operations WHERE order_id = $1 ORDER BY operation_number")
                .bind(h.order_id)
                .fetch_all(&self.pool).await?;
            h.operations = ops;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }

    pub async fn confirm_operation(
        &self,
        order_id: uuid::Uuid,
        op_num: &str,
        actual_duration: rust_decimal::Decimal,
    ) -> Result<()> {
        sqlx::query("UPDATE maintenance_operations SET actual_work_duration = $1, status = 'CNF' WHERE order_id = $2 AND operation_number = $3")
            .bind(actual_duration)
            .bind(order_id)
            .bind(op_num)
        .execute(&self.pool).await?;
        Ok(())
    }
}
