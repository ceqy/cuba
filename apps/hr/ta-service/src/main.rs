use tonic::transport::Server;
use tracing::info;
use dotenvy::dotenv;
use std::sync::Arc;
use cuba_database::{DatabaseConfig, init_pool};

use ta_service::api::grpc_server::TaServiceImpl;
use ta_service::api::proto::hr::ta::v1::talent_acquisition_service_server::TalentAcquisitionServiceServer;
use ta_service::infrastructure::repository::TalentRepository;
use ta_service::application::handlers::TalentHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    dotenv().ok();
    
    let addr = "0.0.0.0:50062".parse()?;
    info!("Starting HR Talent Acquisition Service on {}", addr);

    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    let repo = Arc::new(TalentRepository::new(pool.clone()));
    let handler = Arc::new(TalentHandler::new(repo.clone()));
    let service = TaServiceImpl::new(handler, repo);
    
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(ta_service::api::proto::hr::ta::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("HR Talent Acquisition Service listening on {}", addr);
    
    Server::builder()
        .add_service(TalentAcquisitionServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
