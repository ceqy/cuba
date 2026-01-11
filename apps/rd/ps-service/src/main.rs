use tonic::transport::Server;
use tracing::info;
use dotenvy::dotenv;
use std::sync::Arc;
use cuba_database::{DatabaseConfig, init_pool};

use ps_service::api::grpc_server::PsServiceImpl;
use ps_service::api::proto::rd::ps::v1::project_cost_controlling_service_server::ProjectCostControllingServiceServer;
use ps_service::infrastructure::repository::ProjectCostRepository;
use ps_service::application::handlers::ProjectCostHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    dotenv().ok();
    
    let addr = "0.0.0.0:50068".parse()?;
    info!("Starting RD Project Cost Controlling Service on {}", addr);

    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    let repo = Arc::new(ProjectCostRepository::new(pool.clone()));
    let handler = Arc::new(ProjectCostHandler::new(repo.clone()));
    let service = PsServiceImpl::new(handler, repo);
    
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(ps_service::api::proto::rd::ps::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("RD Project Cost Controlling Service listening on {}", addr);
    
    Server::builder()
        .add_service(ProjectCostControllingServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
