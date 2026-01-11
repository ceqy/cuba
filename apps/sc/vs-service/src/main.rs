use tonic::transport::Server; use tracing::info; use dotenvy::dotenv; use std::sync::Arc; use cuba_database::{DatabaseConfig, init_pool};
use vs_service::api::grpc_server::VsServiceImpl; use vs_service::api::proto::sc::vs::v1::visibility_service_server::VisibilityServiceServer;
use vs_service::infrastructure::repository::VendorRepository; use vs_service::application::handlers::VendorHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry(); dotenv().ok();
    let addr = "0.0.0.0:50087".parse()?;
    info!("Starting SC Visibility Service on {}", addr);
    let db_config = DatabaseConfig::default(); let pool = init_pool(&db_config).await?;
    let migrator = sqlx::migrate!("./migrations"); cuba_database::run_migrations(&pool, &migrator).await?;
    let repo = Arc::new(VendorRepository::new(pool.clone())); let handler = Arc::new(VendorHandler::new(repo.clone())); let service = VsServiceImpl::new(handler, repo);
    let reflection_service = tonic_reflection::server::Builder::configure().register_encoded_file_descriptor_set(vs_service::api::proto::sc::vs::v1::FILE_DESCRIPTOR_SET).build_v1()?;
    Server::builder().add_service(VisibilityServiceServer::new(service)).add_service(reflection_service).serve(addr).await?;
    Ok(())
}
