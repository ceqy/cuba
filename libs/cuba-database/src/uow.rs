use sqlx::{Postgres, Transaction};
use async_trait::async_trait;
use cuba_errors::ServiceError;
use crate::db::PostgresDb;

#[async_trait]
pub trait UnitOfWork: Send + Sync {
    async fn begin(&self) -> Result<Transaction<'static, Postgres>, ServiceError>;
}

#[async_trait]
impl UnitOfWork for PostgresDb {
    async fn begin(&self) -> Result<Transaction<'static, Postgres>, ServiceError> {
        let tx = self.pool().begin().await.map_err(ServiceError::DatabaseError)?;
        Ok(tx)
    }
}
