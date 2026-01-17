use tonic::transport::Server;
use tracing::info;
use std::sync::Arc;

use eh_service::api::grpc_server::EhServiceImpl;
use eh_service::api::proto::am::eh::v1::ehs_incident_service_server::EhsIncidentServiceServer;
use eh_service::infrastructure::repository::IncidentRepository;
use eh_service::application::handlers::IncidentHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50085).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;

    let repo = Arc::new(IncidentRepository::new(pool.clone()));
    let handler = Arc::new(IncidentHandler::new(repo.clone()));
    let service = EhServiceImpl::new(handler, repo);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(eh_service::api::proto::am::eh::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("AM EHS Incident Service listening on {}", addr);

    Server::builder()
        .add_service(EhsIncidentServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
