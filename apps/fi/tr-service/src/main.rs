use tonic::transport::Server;
use tracing::info;
use dotenvy::dotenv;
use std::sync::Arc;
use cuba_database::{DatabaseConfig, init_pool};

use tr_service::api::grpc_server::TrServiceImpl;
use tr_service::api::proto::fi::tr::v1::treasury_service_server::TreasuryServiceServer;
use tr_service::infrastructure::repository::TreasuryRepository;
use tr_service::application::handlers::TreasuryHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    dotenv().ok();
    
    let addr = "0.0.0.0:50071".parse()?;
    info!("Starting FI Treasury Service on {}", addr);

    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    let repo = Arc::new(TreasuryRepository::new(pool.clone()));
    let handler = Arc::new(TreasuryHandler::new(repo.clone()));
    let service = TrServiceImpl::new(handler, repo);
    
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(tr_service::api::proto::fi::tr::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("FI Treasury Service listening on {}", addr);
    
    Server::builder()
        .add_service(TreasuryServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
