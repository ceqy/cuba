use tonic::transport::Server;
use tracing::info;
use std::sync::Arc;
use se_service::api::grpc_server::SeServiceImpl;
use se_service::api::proto::pm::se::v1::sourcing_event_service_server::SourcingEventServiceServer;
use se_service::infrastructure::repository::SourcingRepository;
use se_service::application::handlers::SourcingHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50082).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    let repo = Arc::new(SourcingRepository::new(pool.clone()));
    let handler = Arc::new(SourcingHandler::new(repo.clone()));
    let service = SeServiceImpl::new(handler, repo);
    
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(se_service::api::proto::pm::se::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("PM Sourcing Event Service listening on {}", addr);
    
    Server::builder()
        .add_service(SourcingEventServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
