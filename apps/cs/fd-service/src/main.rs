use fd_service::api::grpc_server::FdServiceImpl;
use fd_service::api::proto::cs::fd::v1::field_service_dispatch_service_server::FieldServiceDispatchServiceServer;
use fd_service::application::handlers::ServiceHandler;
use fd_service::infrastructure::repository::ServiceOrderRepository;
use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50064).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;

    let repo = Arc::new(ServiceOrderRepository::new(pool.clone()));
    let handler = Arc::new(ServiceHandler::new(repo.clone()));
    let service = FdServiceImpl::new(handler, repo);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(
            fd_service::api::proto::cs::fd::v1::FILE_DESCRIPTOR_SET,
        )
        .build_v1()?;

    info!("CS Field Service Dispatch Service listening on {}", addr);

    Server::builder()
        .add_service(FieldServiceDispatchServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
