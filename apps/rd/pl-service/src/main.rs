use pl_service::api::grpc_server::PlServiceImpl;
use pl_service::api::proto::rd::pl::v1::plm_integration_service_server::PlmIntegrationServiceServer;
use pl_service::application::handlers::PLMHandler;
use pl_service::infrastructure::repository::BOMRepository;
use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50066).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;

    let repo = Arc::new(BOMRepository::new(pool.clone()));
    let handler = Arc::new(PLMHandler::new(repo.clone()));
    let service = PlServiceImpl::new(handler, repo);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(
            pl_service::api::proto::rd::pl::v1::FILE_DESCRIPTOR_SET,
        )
        .build_v1()?;

    info!("RD PLM Integration Service listening on {}", addr);

    Server::builder()
        .add_service(PlmIntegrationServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
