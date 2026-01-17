use anyhow::Result;
use cuba_database::{DatabaseConfig, init_pool, DbPool};
use dotenvy::dotenv;

pub struct ServiceContext {
    pub db_pool: DbPool,
    pub addr: std::net::SocketAddr,
}

pub struct ServiceBootstrapper;

impl ServiceBootstrapper {
    pub async fn run(default_port: u16) -> Result<ServiceContext> {
        dotenv().ok(); // Load .env
        cuba_telemetry::init_telemetry();

        // Config
        let port_env = std::env::var("PORT").unwrap_or_else(|_| default_port.to_string());
        let addr = format!("0.0.0.0:{}", port_env).parse()?;
        tracing::info!("Starting service on {}", addr);

        // Database
        let db_config = DatabaseConfig::default();
        let pool = init_pool(&db_config).await?;
        tracing::info!("Connected to database: {}", db_config.url);

        Ok(ServiceContext {
            db_pool: pool,
            addr,
        })
    }
}
