use tonic::transport::Server;
use tracing::info;
use std::sync::Arc;

use tr_service::api::grpc_server::TrServiceImpl;
use tr_service::api::proto::fi::tr::v1::treasury_service_server::TreasuryServiceServer;
use tr_service::infrastructure::repository::TreasuryRepository;
use tr_service::application::handlers::TreasuryHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50064).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;

    // Initialize GL Client using unified function
    let gl_client = cuba_finance::create_gl_client(
        "http://gl-service.cuba-fi.svc.cluster.local:50060"
    ).await?;
    
    let repo = Arc::new(TreasuryRepository::new(pool.clone()));
    let handler = Arc::new(TreasuryHandler::new(repo.clone(), gl_client));
    let service = TrServiceImpl::new(handler, repo);
    
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(tr_service::api::proto::fi::tr::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("FI Treasury Service listening on {}", addr);
    
    Server::builder()
        .add_service(TreasuryServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
