use om_service::api::grpc_server::OmServiceImpl;
use om_service::api::proto::mf::om::v1::outsourced_manufacturing_service_server::OutsourcedManufacturingServiceServer;
use om_service::application::handlers::SubcontractingHandler;
use om_service::infrastructure::repository::SubcontractingRepository;
use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50080).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;

    let repo = Arc::new(SubcontractingRepository::new(pool.clone()));
    let handler = Arc::new(SubcontractingHandler::new(repo.clone()));
    let service = OmServiceImpl::new(handler, repo);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(
            om_service::api::proto::mf::om::v1::FILE_DESCRIPTOR_SET,
        )
        .build_v1()?;

    info!("MF Outsourced Manufacturing Service listening on {}", addr);

    Server::builder()
        .add_service(OutsourcedManufacturingServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
