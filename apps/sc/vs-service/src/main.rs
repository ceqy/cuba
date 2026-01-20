use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

use vs_service::api::grpc_server::VsServiceImpl;
use vs_service::api::proto::sc::vs::v1::visibility_service_server::VisibilityServiceServer;
use vs_service::application::handlers::VendorHandler;
use vs_service::infrastructure::repository::VendorRepository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50087).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;

    let repo = Arc::new(VendorRepository::new(pool.clone()));
    let handler = Arc::new(VendorHandler::new(repo.clone()));
    let service = VsServiceImpl::new(handler, repo);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(
            vs_service::api::proto::sc::vs::v1::FILE_DESCRIPTOR_SET,
        )
        .build_v1()?;

    info!("SC Visibility Service listening on {}", addr);

    Server::builder()
        .add_service(VisibilityServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
