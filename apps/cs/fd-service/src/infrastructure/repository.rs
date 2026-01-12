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
        sqlx::query(
            "INSERT INTO service_orders (order_id, order_number, order_type, customer_id, description, planned_start, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
            .bind(o.order_id)
            .bind(&o.order_number)
            .bind(&o.order_type)
            .bind(&o.customer_id)
            .bind(&o.description)
            .bind(o.planned_start)
            .bind(&o.status)
            .bind(o.created_at)
            .bind(o.updated_at)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn find_by_number(&self, order_number: &str) -> Result<Option<ServiceOrder>> {
        let r = sqlx::query_as::<_, ServiceOrder>("SELECT order_id, order_number, order_type, customer_id, description, planned_start, assigned_technician_id, status, created_at, updated_at FROM service_orders WHERE order_number = $1")
            .bind(order_number)
            .fetch_optional(&self.pool).await?;
        Ok(r)
    }

    pub async fn assign_technician(&self, order_number: &str, tech_id: &str, scheduled: DateTime<Utc>) -> Result<()> {
        sqlx::query("UPDATE service_orders SET assigned_technician_id = $1, planned_start = $2, status = 'ASSIGNED', updated_at = NOW() WHERE order_number = $3")
            .bind(tech_id)
            .bind(scheduled)
            .bind(order_number)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn update_status(&self, order_number: &str, status: &str) -> Result<()> {
        sqlx::query("UPDATE service_orders SET status = $1, updated_at = NOW() WHERE order_number = $2")
            .bind(status)
            .bind(order_number)
        .execute(&self.pool).await?;
        Ok(())
    }
}
