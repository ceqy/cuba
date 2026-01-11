use tonic::transport::Server; use tracing::info; use dotenvy::dotenv; use std::sync::Arc; use cuba_database::{DatabaseConfig, init_pool};
use eh_service::api::grpc_server::EhServiceImpl; use eh_service::api::proto::am::eh::v1::ehs_incident_service_server::EhsIncidentServiceServer;
use eh_service::infrastructure::repository::IncidentRepository; use eh_service::application::handlers::IncidentHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry(); dotenv().ok();
    let addr = "0.0.0.0:50085".parse()?;
    info!("Starting AM EHS Incident Service on {}", addr);
    let db_config = DatabaseConfig::default(); let pool = init_pool(&db_config).await?;
    let migrator = sqlx::migrate!("./migrations"); cuba_database::run_migrations(&pool, &migrator).await?;
    let repo = Arc::new(IncidentRepository::new(pool.clone())); let handler = Arc::new(IncidentHandler::new(repo.clone())); let service = EhServiceImpl::new(handler, repo);
    let reflection_service = tonic_reflection::server::Builder::configure().register_encoded_file_descriptor_set(eh_service::api::proto::am::eh::v1::FILE_DESCRIPTOR_SET).build_v1()?;
    Server::builder().add_service(EhsIncidentServiceServer::new(service)).add_service(reflection_service).serve(addr).await?;
    Ok(())
}
