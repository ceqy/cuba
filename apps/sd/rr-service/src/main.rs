use rr_service::api::grpc_server::RrServiceImpl;
use rr_service::api::proto::sd::rr::v1::revenue_recognition_service_server::RevenueRecognitionServiceServer;
use rr_service::application::handlers::RevenueHandler;
use rr_service::infrastructure::repository::RevenueRepository;
use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50078).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;

    let repo = Arc::new(RevenueRepository::new(pool.clone()));
    let handler = Arc::new(RevenueHandler::new(repo.clone()));
    let service = RrServiceImpl::new(handler, repo);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(
            rr_service::api::proto::sd::rr::v1::FILE_DESCRIPTOR_SET,
        )
        .build_v1()?;

    info!("SD Revenue Recognition Service listening on {}", addr);

    Server::builder()
        .add_service(RevenueRecognitionServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
