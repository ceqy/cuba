use tonic::transport::Server;
use tracing::info;
use dotenvy::dotenv;
use std::sync::Arc;
use cuba_database::{DatabaseConfig, init_pool};

use wc_service::api::grpc_server::WcServiceImpl;
use wc_service::api::proto::cs::wc::v1::warranty_claims_service_server::WarrantyClaimsServiceServer;
use wc_service::infrastructure::repository::ClaimRepository;
use wc_service::application::handlers::ClaimHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    dotenv().ok();
    
    let addr = "0.0.0.0:50072".parse()?;
    info!("Starting CS Warranty Claims Service on {}", addr);

    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    let repo = Arc::new(ClaimRepository::new(pool.clone()));
    let handler = Arc::new(ClaimHandler::new(repo.clone()));
    let service = WcServiceImpl::new(handler, repo);
    
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(wc_service::api::proto::cs::wc::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("CS Warranty Claims Service listening on {}", addr);
    
    Server::builder()
        .add_service(WarrantyClaimsServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
