use tonic::transport::Server;
use tracing::info;
use dotenvy::dotenv;
use std::sync::Arc;
use cuba_database::{DatabaseConfig, init_pool};

use cb_service::api::grpc_server::CbServiceImpl;
use cb_service::api::proto::cs::cb::v1::contract_billing_service_server::ContractBillingServiceServer;
use cb_service::infrastructure::repository::ContractRepository;
use cb_service::application::handlers::BillingHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    dotenv().ok();
    
    let addr = "0.0.0.0:50065".parse()?;
    info!("Starting CS Contract Billing Service on {}", addr);

    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    let repo = Arc::new(ContractRepository::new(pool.clone()));
    let handler = Arc::new(BillingHandler::new(repo.clone()));
    let service = CbServiceImpl::new(handler, repo);
    
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(cb_service::api::proto::cs::cb::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("CS Contract Billing Service listening on {}", addr);
    
    Server::builder()
        .add_service(ContractBillingServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
