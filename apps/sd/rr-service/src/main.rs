use tonic::transport::Server;
use tracing::info;
use dotenvy::dotenv;
use std::sync::Arc;
use cuba_database::{DatabaseConfig, init_pool};

use rr_service::api::grpc_server::RrServiceImpl;
use rr_service::api::proto::sd::rr::v1::revenue_recognition_service_server::RevenueRecognitionServiceServer;
use rr_service::infrastructure::repository::RevenueRepository;
use rr_service::application::handlers::RevenueHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    dotenv().ok();
    
    let addr = "0.0.0.0:50078".parse()?;
    info!("Starting SD Revenue Recognition Service on {}", addr);

    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    let repo = Arc::new(RevenueRepository::new(pool.clone()));
    let handler = Arc::new(RevenueHandler::new(repo.clone()));
    let service = RrServiceImpl::new(handler, repo);
    
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(rr_service::api::proto::sd::rr::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("SD Revenue Recognition Service listening on {}", addr);
    
    Server::builder()
        .add_service(RevenueRecognitionServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
