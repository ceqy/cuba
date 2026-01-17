use tonic::transport::Server;
use tracing::info;
use std::sync::Arc;
use iv_service::api::grpc_server::IvServiceImpl;
use iv_service::api::proto::pm::iv::v1::invoice_processing_service_server::InvoiceProcessingServiceServer;
use iv_service::infrastructure::repository::InvoiceRepository;
use iv_service::application::handlers::InvoiceHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50069).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    let repo = Arc::new(InvoiceRepository::new(pool.clone()));
    let handler = Arc::new(InvoiceHandler::new(repo.clone()));
    let service = IvServiceImpl::new(handler, repo);
    
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(iv_service::api::proto::pm::iv::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("PM Invoice Processing Service listening on {}", addr);
    
    Server::builder()
        .add_service(InvoiceProcessingServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
