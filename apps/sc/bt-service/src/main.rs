use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

use bt_service::api::grpc_server::BtServiceImpl;
use bt_service::api::proto::sc::bt::v1::batch_traceability_service_server::BatchTraceabilityServiceServer;
use bt_service::application::handlers::BatchHandler;
use bt_service::infrastructure::repository::BatchRepository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50073).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;

    let repo = Arc::new(BatchRepository::new(pool.clone()));
    let handler = Arc::new(BatchHandler::new(repo.clone()));
    let service = BtServiceImpl::new(handler, repo);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(
            bt_service::api::proto::sc::bt::v1::FILE_DESCRIPTOR_SET,
        )
        .build_v1()?;

    info!("SC Batch Traceability Service listening on {}", addr);

    Server::builder()
        .add_service(BatchTraceabilityServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
