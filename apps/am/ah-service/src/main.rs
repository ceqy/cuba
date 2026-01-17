use tonic::transport::Server;
use tracing::info;
use std::sync::Arc;

use ah_service::api::grpc_server::AhServiceImpl;
use ah_service::api::proto::am::ah::v1::intelligent_asset_health_service_server::IntelligentAssetHealthServiceServer;
use ah_service::infrastructure::repository::HealthRepository;
use ah_service::application::handlers::HealthHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50083).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    let repo = Arc::new(HealthRepository::new(pool.clone()));
    let handler = Arc::new(HealthHandler::new(repo.clone()));
    let service = AhServiceImpl::new(handler, repo);
    
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(ah_service::api::proto::am::ah::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("AM Intelligent Asset Health Service listening on {}", addr);
    
    Server::builder()
        .add_service(IntelligentAssetHealthServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
