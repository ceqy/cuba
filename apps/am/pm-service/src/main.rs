use am_pm_service::api::grpc_server::PmServiceImpl;
use am_pm_service::api::proto::am::pm::v1::asset_maintenance_service_server::AssetMaintenanceServiceServer;
use am_pm_service::application::handlers::MaintenanceHandler;
use am_pm_service::infrastructure::repository::MaintenanceRepository;
use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50061).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;

    let repo = Arc::new(MaintenanceRepository::new(pool.clone()));
    let handler = Arc::new(MaintenanceHandler::new(repo.clone()));
    let service = PmServiceImpl::new(handler, repo);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(
            am_pm_service::api::proto::am::pm::v1::FILE_DESCRIPTOR_SET,
        )
        .build_v1()?;

    info!("AM Plant Maintenance Service listening on {}", addr);

    Server::builder()
        .add_service(AssetMaintenanceServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
