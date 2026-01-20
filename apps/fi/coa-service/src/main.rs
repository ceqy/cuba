// COA Service Main Entry Point
use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

use coa_service::api::grpc_server::CoaGrpcService;
use coa_service::application::CoaApplicationService;
use coa_service::infrastructure::PgGlAccountRepository;
use coa_service::infrastructure::grpc::chart_of_accounts_service_server::ChartOfAccountsServiceServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50065).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("Migrations completed");

    // Initialize layers
    let repository = Arc::new(PgGlAccountRepository::new(pool.clone()));
    let app_service = Arc::new(CoaApplicationService::new(repository));
    let grpc_service = CoaGrpcService::new(app_service);

    info!("COA Service listening on {}", addr);

    // Reflection Service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(
            coa_service::infrastructure::grpc::fi::coa::v1::FILE_DESCRIPTOR_SET,
        )
        .build_v1()?;

    // Start gRPC server
    Server::builder()
        .add_service(reflection_service)
        .add_service(ChartOfAccountsServiceServer::new(grpc_service))
        .serve(addr)
        .await?;

    Ok(())
}
