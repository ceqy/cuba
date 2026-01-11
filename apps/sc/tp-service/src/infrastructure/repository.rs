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
        sqlx::query!(
            "INSERT INTO shipments (shipment_id, shipment_number, shipment_type, transportation_planning_point, carrier, overall_status, planned_departure, planned_arrival) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            s.shipment_id, s.shipment_number, s.shipment_type, s.transportation_planning_point, s.carrier, s.overall_status, s.planned_departure, s.planned_arrival
        ).execute(&mut *tx).await?;
        for item in &s.items {
            sqlx::query!(
                "INSERT INTO shipment_items (item_id, shipment_id, item_number, delivery_number, total_weight, weight_unit, volume, volume_unit) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                item.item_id, item.shipment_id, item.item_number, item.delivery_number, item.total_weight, item.weight_unit, item.volume, item.volume_unit
            ).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_number(&self, num: &str) -> Result<Option<Shipment>> {
        let h = sqlx::query!("SELECT * FROM shipments WHERE shipment_number = $1", num)
            .fetch_optional(&self.pool).await?;
        if let Some(h) = h {
            let items = sqlx::query!("SELECT * FROM shipment_items WHERE shipment_id = $1", h.shipment_id)
                .fetch_all(&self.pool).await?;
            Ok(Some(Shipment {
                shipment_id: h.shipment_id,
                shipment_number: h.shipment_number,
                shipment_type: h.shipment_type,
                transportation_planning_point: h.transportation_planning_point,
                carrier: h.carrier,
                overall_status: h.overall_status.unwrap_or_default(),
                planned_departure: h.planned_departure,
                planned_arrival: h.planned_arrival,
                created_at: h.created_at,
                items: items.into_iter().map(|i| ShipmentItem {
                    item_id: i.item_id,
                    shipment_id: i.shipment_id,
                    item_number: i.item_number,
                    delivery_number: i.delivery_number,
                    total_weight: i.total_weight,
                    weight_unit: i.weight_unit.unwrap_or_default(),
                    volume: i.volume,
                    volume_unit: i.volume_unit.unwrap_or_default(),
                }).collect(),
            }))
        } else { Ok(None) }
    }

    pub async fn update_status(&self, num: &str, status: &str) -> Result<()> {
        sqlx::query!("UPDATE shipments SET overall_status = $1 WHERE shipment_number = $2", status, num)
            .execute(&self.pool).await?;
        Ok(())
    }
}
