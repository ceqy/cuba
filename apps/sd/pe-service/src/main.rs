use pe_service::api::grpc_server::PeServiceImpl;
use pe_service::api::proto::sd::pe::v1::pricing_engine_service_server::PricingEngineServiceServer;
use pe_service::application::handlers::PricingHandler;
use pe_service::infrastructure::repository::PricingRepository;
use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50075).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;

    let repo = Arc::new(PricingRepository::new(pool.clone()));
    let handler = Arc::new(PricingHandler::new(repo));
    let service = PeServiceImpl::new(handler);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(
            pe_service::api::proto::sd::pe::v1::FILE_DESCRIPTOR_SET,
        )
        .build_v1()?;

    info!("SD Pricing Engine Service listening on {}", addr);

    Server::builder()
        .add_service(PricingEngineServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
