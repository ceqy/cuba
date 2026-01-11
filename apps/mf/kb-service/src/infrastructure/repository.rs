use sqlx::PgPool;
use crate::domain::{ControlCycle, KanbanContainer};
use anyhow::Result;

pub struct KanbanRepository {
    pool: PgPool,
}

impl KanbanRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_cycle(&self, c: &ControlCycle) -> Result<()> {
        sqlx::query!(
            "INSERT INTO control_cycles (cycle_id, cycle_number, material, plant, supply_area, number_of_kanbans, qty_per_kanban, unit, replenishment_strategy) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            c.cycle_id, c.cycle_number, c.material, c.plant, c.supply_area, c.number_of_kanbans, c.qty_per_kanban, c.unit, c.replenishment_strategy
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn save_container(&self, k: &KanbanContainer) -> Result<()> {
        sqlx::query!(
            "INSERT INTO kanban_containers (container_id, container_code, cycle_id, status) VALUES ($1, $2, $3, $4)",
            k.container_id, k.container_code, k.cycle_id, k.status
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn find_cycle_by_number(&self, num: &str) -> Result<Option<ControlCycle>> {
        let h = sqlx::query!("SELECT * FROM control_cycles WHERE cycle_number = $1", num)
            .fetch_optional(&self.pool).await?;
        Ok(h.map(|h| ControlCycle {
            cycle_id: h.cycle_id,
            cycle_number: h.cycle_number,
            material: h.material,
            plant: h.plant,
            supply_area: h.supply_area,
            number_of_kanbans: h.number_of_kanbans.unwrap_or(3),
            qty_per_kanban: h.qty_per_kanban.unwrap_or_default(),
            unit: h.unit.unwrap_or_default(),
            replenishment_strategy: h.replenishment_strategy.unwrap_or_default(),
            created_at: h.created_at,
        }))
    }

    pub async fn find_container_by_code(&self, code: &str) -> Result<Option<KanbanContainer>> {
        let h = sqlx::query!("SELECT * FROM kanban_containers WHERE container_code = $1", code)
            .fetch_optional(&self.pool).await?;
        Ok(h.map(|h| KanbanContainer {
            container_id: h.container_id,
            container_code: h.container_code,
            cycle_id: h.cycle_id,
            status: h.status.unwrap_or_default(),
            last_status_change: h.last_status_change.unwrap_or_else(|| chrono::Utc::now()),
        }))
    }

    pub async fn update_container_status(&self, container_id: uuid::Uuid, new_status: &str) -> Result<()> {
        sqlx::query!("UPDATE kanban_containers SET status = $1, last_status_change = NOW() WHERE container_id = $2", new_status, container_id)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn list_containers_by_cycle(&self, cycle_id: uuid::Uuid) -> Result<Vec<KanbanContainer>> {
        let rows = sqlx::query!("SELECT * FROM kanban_containers WHERE cycle_id = $1", cycle_id)
            .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|h| KanbanContainer {
            container_id: h.container_id,
            container_code: h.container_code,
            cycle_id: h.cycle_id,
            status: h.status.unwrap_or_default(),
            last_status_change: h.last_status_change.unwrap_or_else(|| chrono::Utc::now()),
        }).collect())
    }
}
