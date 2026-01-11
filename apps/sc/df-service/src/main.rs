use tonic::transport::Server;
use tracing::info;
use dotenvy::dotenv;
use std::sync::Arc;
use cuba_database::{DatabaseConfig, init_pool};

use df_service::api::grpc_server::DfServiceImpl;
use df_service::api::proto::sc::df::v1::demand_forecasting_service_server::DemandForecastingServiceServer;
use df_service::infrastructure::repository::ForecastRepository;
use df_service::application::handlers::ForecastHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    dotenv().ok();
    
    let addr = "0.0.0.0:50081".parse()?;
    info!("Starting SC Demand Forecasting Service on {}", addr);

    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    let repo = Arc::new(ForecastRepository::new(pool.clone()));
    let handler = Arc::new(ForecastHandler::new(repo.clone()));
    let service = DfServiceImpl::new(handler, repo);
    
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(df_service::api::proto::sc::df::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("SC Demand Forecasting Service listening on {}", addr);
    
    Server::builder()
        .add_service(DemandForecastingServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
