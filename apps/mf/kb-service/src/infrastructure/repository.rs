use crate::domain::{ControlCycle, KanbanContainer};
use anyhow::Result;
use sqlx::PgPool;

pub struct KanbanRepository {
    pool: PgPool,
}

impl KanbanRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_cycle(&self, c: &ControlCycle) -> Result<()> {
        sqlx::query(
            "INSERT INTO control_cycles (cycle_id, cycle_number, material, plant, supply_area, number_of_kanbans, qty_per_kanban, unit, replenishment_strategy) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
            .bind(c.cycle_id)
            .bind(&c.cycle_number)
            .bind(&c.material)
            .bind(&c.plant)
            .bind(&c.supply_area)
            .bind(c.number_of_kanbans)
            .bind(c.qty_per_kanban)
            .bind(&c.unit)
            .bind(&c.replenishment_strategy)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn save_container(&self, k: &KanbanContainer) -> Result<()> {
        sqlx::query(
            "INSERT INTO kanban_containers (container_id, container_code, cycle_id, status) VALUES ($1, $2, $3, $4)")
            .bind(k.container_id)
            .bind(&k.container_code)
            .bind(k.cycle_id)
            .bind(&k.status)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn find_cycle_by_number(&self, num: &str) -> Result<Option<ControlCycle>> {
        let h = sqlx::query_as::<_, ControlCycle>("SELECT cycle_id, cycle_number, material, plant, supply_area, number_of_kanbans, qty_per_kanban, unit, replenishment_strategy, created_at FROM control_cycles WHERE cycle_number = $1")
            .bind(num)
            .fetch_optional(&self.pool).await?;
        Ok(h)
    }

    pub async fn find_container_by_code(&self, code: &str) -> Result<Option<KanbanContainer>> {
        let h = sqlx::query_as::<_, KanbanContainer>("SELECT container_id, container_code, cycle_id, status, last_status_change FROM kanban_containers WHERE container_code = $1")
            .bind(code)
            .fetch_optional(&self.pool).await?;
        Ok(h)
    }

    pub async fn update_container_status(
        &self,
        container_id: uuid::Uuid,
        new_status: &str,
    ) -> Result<()> {
        sqlx::query("UPDATE kanban_containers SET status = $1, last_status_change = NOW() WHERE container_id = $2")
            .bind(new_status)
            .bind(container_id)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn list_containers_by_cycle(
        &self,
        cycle_id: uuid::Uuid,
    ) -> Result<Vec<KanbanContainer>> {
        let rows = sqlx::query_as::<_, KanbanContainer>("SELECT container_id, container_code, cycle_id, status, last_status_change FROM kanban_containers WHERE cycle_id = $1")
            .bind(cycle_id)
            .fetch_all(&self.pool).await?;
        Ok(rows)
    }
}
