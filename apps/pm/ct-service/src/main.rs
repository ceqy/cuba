use ct_service::api::grpc_server::CtServiceImpl;
use ct_service::api::proto::pm::ct::v1::contract_management_service_server::ContractManagementServiceServer;
use ct_service::application::handlers::ContractHandler;
use ct_service::infrastructure::repository::ContractRepository;
use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50076).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;

    let repo = Arc::new(ContractRepository::new(pool.clone()));
    let handler = Arc::new(ContractHandler::new(repo.clone()));
    let service = CtServiceImpl::new(handler, repo);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(
            ct_service::api::proto::pm::ct::v1::FILE_DESCRIPTOR_SET,
        )
        .build_v1()?;

    info!("PM Contract Management Service listening on {}", addr);

    Server::builder()
        .add_service(ContractManagementServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
