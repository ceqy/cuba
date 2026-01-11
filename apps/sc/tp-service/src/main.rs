use tonic::transport::Server;
use tracing::info;
use dotenvy::dotenv;
use std::sync::Arc;
use cuba_database::{DatabaseConfig, init_pool};

use tp_service::api::grpc_server::TpServiceImpl;
use tp_service::api::proto::sc::tp::v1::transportation_planning_service_server::TransportationPlanningServiceServer;
use tp_service::infrastructure::repository::ShipmentRepository;
use tp_service::application::handlers::ShipmentHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    dotenv().ok();
    let addr = "0.0.0.0:50084".parse()?;
    info!("Starting SC Transportation Planning Service on {}", addr);
    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;
    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    let repo = Arc::new(ShipmentRepository::new(pool.clone()));
    let handler = Arc::new(ShipmentHandler::new(repo.clone()));
    let service = TpServiceImpl::new(handler, repo);
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(tp_service::api::proto::sc::tp::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;
    Server::builder()
        .add_service(TransportationPlanningServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr).await?;
    Ok(())
}
