use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::postgres::{PgPool, PgPoolOptions, PgTransaction};
use sqlx::Transaction;
use std::time::Duration;
use tracing::info;

pub type DbPool = PgPool;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub idle_timeout: Duration,
    pub connect_timeout: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: std::env::var("DATABASE_URL").unwrap_or_default(),
            max_connections: 10,
            min_connections: 1,
            idle_timeout: Duration::from_secs(600),
            connect_timeout: Duration::from_secs(10),
        }
    }
}

pub async fn init_pool(config: &DatabaseConfig) -> Result<DbPool> {
    info!("Initializing database connection pool...");
    
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .idle_timeout(config.idle_timeout)
        .acquire_timeout(config.connect_timeout)
        .connect(&config.url)
        .await
        .context("Failed to connect to database")?;

    info!("Database connection pool initialized successfully");
    Ok(pool)
}

/// UnitOfWork trait for transaction management
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    async fn begin(&self) -> Result<sqlx::Transaction<'static, sqlx::Postgres>>;
}

#[async_trait]
impl UnitOfWork for DbPool {
    async fn begin(&self) -> Result<sqlx::Transaction<'static, sqlx::Postgres>> {
        let tx = self.begin().await.context("Failed to begin transaction")?;
        Ok(tx)
    }
}

/// Execute a block of code within a transaction
pub async fn with_transaction<F, Fut, T>(pool: &DbPool, f: F) -> Result<T>
where
    F: FnOnce(sqlx::Transaction<'static, sqlx::Postgres>) -> Fut + Send,
    Fut: std::future::Future<Output = Result<(T, sqlx::Transaction<'static, sqlx::Postgres>)>> + Send,
{
    let tx = pool.begin().await.context("Failed to start transaction")?;
    
    match f(tx).await {
        Ok((result, tx)) => {
            tx.commit().await.context("Failed to commit transaction")?;
            Ok(result)
        }
        Err(e) => {
            // Rollback is automatic on drop usually, but explicit requires ownership
            // sqlx transaction rollback handled by Drop if not committed
            Err(e)
        },
    }
}
