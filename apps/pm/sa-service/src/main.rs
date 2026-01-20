use sa_service::api::grpc_server::SaServiceImpl;
use sa_service::api::proto::pm::sa::v1::spend_analytics_service_server::SpendAnalyticsServiceServer;
use sa_service::application::handlers::SpendHandler;
use sa_service::infrastructure::repository::SpendRepository;
use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50077).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;

    let repo = Arc::new(SpendRepository::new(pool.clone()));
    let handler = Arc::new(SpendHandler::new(repo));
    let service = SaServiceImpl::new(handler);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(
            sa_service::api::proto::pm::sa::v1::FILE_DESCRIPTOR_SET,
        )
        .build_v1()?;

    info!("PM Spend Analytics Service listening on {}", addr);

    Server::builder()
        .add_service(SpendAnalyticsServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
