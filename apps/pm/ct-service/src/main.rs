use tonic::transport::Server;
use tracing::info;
use dotenvy::dotenv;
use std::sync::Arc;
use cuba_database::{DatabaseConfig, init_pool};

use ct_service::api::grpc_server::CtServiceImpl;
use ct_service::api::proto::pm::ct::v1::contract_management_service_server::ContractManagementServiceServer;
use ct_service::infrastructure::repository::ContractRepository;
use ct_service::application::handlers::ContractHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    dotenv().ok();
    
    let addr = "0.0.0.0:50076".parse()?;
    info!("Starting PM Contract Management Service on {}", addr);

    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    let repo = Arc::new(ContractRepository::new(pool.clone()));
    let handler = Arc::new(ContractHandler::new(repo.clone()));
    let service = CtServiceImpl::new(handler, repo);
    
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(ct_service::api::proto::pm::ct::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("PM Contract Management Service listening on {}", addr);
    
    Server::builder()
        .add_service(ContractManagementServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
