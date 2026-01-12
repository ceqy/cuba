use sqlx::PgPool;
use crate::domain::{Shipment, ShipmentItem};
use anyhow::Result;

pub struct ShipmentRepository {
    pool: PgPool,
}

impl ShipmentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, s: &Shipment) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO shipments (shipment_id, shipment_number, shipment_type, transportation_planning_point, carrier, overall_status, planned_departure, planned_arrival) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
            .bind(s.shipment_id)
            .bind(&s.shipment_number)
            .bind(&s.shipment_type)
            .bind(&s.transportation_planning_point)
            .bind(&s.carrier)
            .bind(&s.overall_status)
            .bind(s.planned_departure)
            .bind(s.planned_arrival)
        .execute(&mut *tx).await?;
        for item in &s.items {
            sqlx::query(
                "INSERT INTO shipment_items (item_id, shipment_id, item_number, delivery_number, total_weight, weight_unit, volume, volume_unit) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
                .bind(item.item_id)
                .bind(item.shipment_id)
                .bind(item.item_number)
                .bind(&item.delivery_number)
                .bind(item.total_weight)
                .bind(&item.weight_unit)
                .bind(item.volume)
                .bind(&item.volume_unit)
            .execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_number(&self, num: &str) -> Result<Option<Shipment>> {
        let h = sqlx::query_as::<_, Shipment>("SELECT shipment_id, shipment_number, shipment_type, transportation_planning_point, carrier, overall_status, planned_departure, planned_arrival, created_at FROM shipments WHERE shipment_number = $1")
            .bind(num)
            .fetch_optional(&self.pool).await?;
        if let Some(mut h) = h {
            let items = sqlx::query_as::<_, ShipmentItem>("SELECT * FROM shipment_items WHERE shipment_id = $1")
                .bind(h.shipment_id)
                .fetch_all(&self.pool).await?;
            h.items = items;
            Ok(Some(h))
        } else { Ok(None) }
    }

    pub async fn update_status(&self, num: &str, status: &str) -> Result<()> {
        sqlx::query("UPDATE shipments SET overall_status = $1 WHERE shipment_number = $2")
            .bind(status)
            .bind(num)
            .execute(&self.pool).await?;
        Ok(())
    }
}
