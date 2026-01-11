use sqlx::PgPool;
use crate::domain::ServiceOrder;
use anyhow::Result;
use chrono::{DateTime, Utc};

pub struct ServiceOrderRepository {
    pool: PgPool,
}

impl ServiceOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, o: &ServiceOrder) -> Result<()> {
        sqlx::query!(
            "INSERT INTO service_orders (order_id, order_number, order_type, customer_id, description, planned_start, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            o.order_id, o.order_number, o.order_type, o.customer_id, o.description, o.planned_start, o.status, o.created_at, o.updated_at
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn find_by_number(&self, order_number: &str) -> Result<Option<ServiceOrder>> {
        let r = sqlx::query!("SELECT * FROM service_orders WHERE order_number = $1", order_number)
            .fetch_optional(&self.pool).await?;
        Ok(r.map(|r| ServiceOrder {
            order_id: r.order_id,
            order_number: r.order_number,
            order_type: r.order_type.unwrap_or_else(|| "REPAIR".to_string()),
            customer_id: r.customer_id,
            description: r.description,
            planned_start: r.planned_start,
            assigned_technician_id: r.assigned_technician_id,
            status: r.status.unwrap_or_else(|| "OPEN".to_string()),
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    pub async fn assign_technician(&self, order_number: &str, tech_id: &str, scheduled: DateTime<Utc>) -> Result<()> {
        sqlx::query!("UPDATE service_orders SET assigned_technician_id = $1, planned_start = $2, status = 'ASSIGNED', updated_at = NOW() WHERE order_number = $3",
            tech_id, scheduled, order_number
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn update_status(&self, order_number: &str, status: &str) -> Result<()> {
        sqlx::query!("UPDATE service_orders SET status = $1, updated_at = NOW() WHERE order_number = $2",
            status, order_number
        ).execute(&self.pool).await?;
        Ok(())
    }
}
