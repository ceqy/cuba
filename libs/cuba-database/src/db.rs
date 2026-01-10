use sqlx::postgres::{PgPool, PgPoolOptions};
use cuba_errors::ServiceError;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct PostgresDb {
    pool: PgPool,
}

impl PostgresDb {
    pub async fn new(connection_string: &str) -> Result<Self, ServiceError> {
        let pool = PgPoolOptions::new()
            .max_connections(50)
            .min_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            .idle_timeout(Duration::from_secs(600))
            .connect(connection_string)
            .await
            .map_err(ServiceError::DatabaseError)?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn health_check(&self) -> Result<(), ServiceError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(ServiceError::DatabaseError)?;
        Ok(())
    }
}
