use sqlx::PgPool;
use crate::domain::{MaintenanceNotification, MaintenanceOrder, MaintenanceOperation};
use anyhow::Result;

pub struct MaintenanceRepository {
    pool: PgPool,
}

impl MaintenanceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_notification(&self, n: &MaintenanceNotification) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO maintenance_notifications (
                notification_id, notification_number, notification_type,
                description, equipment_number, functional_location,
                reported_by, reported_date, priority, status, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            n.notification_id, n.notification_number, n.notification_type,
            n.description, n.equipment_number, n.functional_location,
            n.reported_by, n.reported_date, n.priority, n.status,
            n.created_at, n.updated_at
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn create_order(&self, o: &MaintenanceOrder) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            r#"
            INSERT INTO maintenance_orders (
                order_id, order_number, order_type, description,
                notification_number, equipment_number, functional_location,
                maintenance_plant, planning_plant, main_work_center,
                system_status, priority, basic_start_date, basic_finish_date,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            "#,
            o.order_id, o.order_number, o.order_type, o.description,
            o.notification_number, o.equipment_number, o.functional_location,
            o.maintenance_plant, o.planning_plant, o.main_work_center,
            o.system_status, o.priority, o.basic_start_date, o.basic_finish_date,
            o.created_at, o.updated_at
        ).execute(&mut *tx).await?;

        for op in &o.operations {
            sqlx::query!(
                r#"
                INSERT INTO maintenance_operations (
                    operation_id, order_id, operation_number, description,
                    work_center, planned_work_duration, actual_work_duration,
                    work_unit, status
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
                op.operation_id, op.order_id, op.operation_number, op.description,
                op.work_center, op.planned_work_duration, op.actual_work_duration,
                op.work_unit, op.status
            ).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_order_by_number(&self, order_number: &str) -> Result<Option<MaintenanceOrder>> {
        let h = sqlx::query!(
            r#"SELECT * FROM maintenance_orders WHERE order_number = $1"#,
            order_number
        ).fetch_optional(&self.pool).await?;

        if let Some(h) = h {
            let ops = sqlx::query!(
                r#"SELECT * FROM maintenance_operations WHERE order_id = $1 ORDER BY operation_number"#,
                h.order_id
            ).fetch_all(&self.pool).await?;

            Ok(Some(MaintenanceOrder {
                order_id: h.order_id,
                order_number: h.order_number,
                order_type: h.order_type,
                description: h.description,
                notification_number: h.notification_number,
                equipment_number: h.equipment_number,
                functional_location: h.functional_location,
                maintenance_plant: h.maintenance_plant,
                planning_plant: h.planning_plant,
                main_work_center: h.main_work_center,
                system_status: h.system_status.unwrap_or_else(|| "CRTD".to_string()),
                priority: h.priority.unwrap_or_else(|| "3".to_string()),
                basic_start_date: h.basic_start_date,
                basic_finish_date: h.basic_finish_date,
                created_at: h.created_at,
                updated_at: h.updated_at,
                operations: ops.into_iter().map(|op| MaintenanceOperation {
                    operation_id: op.operation_id,
                    order_id: op.order_id,
                    operation_number: op.operation_number,
                    description: op.description,
                    work_center: op.work_center,
                    planned_work_duration: op.planned_work_duration.unwrap_or_default(),
                    actual_work_duration: op.actual_work_duration.unwrap_or_default(),
                    work_unit: op.work_unit.unwrap_or_else(|| "H".to_string()),
                    status: op.status.unwrap_or_else(|| "CRTD".to_string()),
                }).collect(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn confirm_operation(&self, order_id: uuid::Uuid, op_num: &str, actual_duration: rust_decimal::Decimal) -> Result<()> {
        sqlx::query!(
            r#"UPDATE maintenance_operations SET actual_work_duration = $1, status = 'CNF' WHERE order_id = $2 AND operation_number = $3"#,
            actual_duration, order_id, op_num
        ).execute(&self.pool).await?;
        Ok(())
    }
}
