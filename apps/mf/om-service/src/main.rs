use tonic::transport::Server;
use tracing::info;
use dotenvy::dotenv;
use std::sync::Arc;
use cuba_database::{DatabaseConfig, init_pool};

use om_service::api::grpc_server::OmServiceImpl;
use om_service::api::proto::mf::om::v1::outsourced_manufacturing_service_server::OutsourcedManufacturingServiceServer;
use om_service::infrastructure::repository::SubcontractingRepository;
use om_service::application::handlers::SubcontractingHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    dotenv().ok();
    
    let addr = "0.0.0.0:50080".parse()?;
    info!("Starting MF Outsourced Manufacturing Service on {}", addr);

    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    let repo = Arc::new(SubcontractingRepository::new(pool.clone()));
    let handler = Arc::new(SubcontractingHandler::new(repo.clone()));
    let service = OmServiceImpl::new(handler, repo);
    
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(om_service::api::proto::mf::om::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("MF Outsourced Manufacturing Service listening on {}", addr);
    
    Server::builder()
        .add_service(OutsourcedManufacturingServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
