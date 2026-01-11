use tonic::transport::Server; use tracing::info; use dotenvy::dotenv; use std::sync::Arc; use cuba_database::{DatabaseConfig, init_pool};
use gs_service::api::grpc_server::GsServiceImpl; use gs_service::api::proto::am::gs::v1::geo_service_server::GeoServiceServer;
use gs_service::infrastructure::repository::SettingsRepository; use gs_service::application::handlers::SettingsHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry(); dotenv().ok();
    let addr = "0.0.0.0:50086".parse()?;
    info!("Starting AM Geo Service on {}", addr);
    let db_config = DatabaseConfig::default(); let pool = init_pool(&db_config).await?;
    let migrator = sqlx::migrate!("./migrations"); cuba_database::run_migrations(&pool, &migrator).await?;
    let repo = Arc::new(SettingsRepository::new(pool.clone())); let handler = Arc::new(SettingsHandler::new(repo.clone())); let service = GsServiceImpl::new(handler, repo);
    let reflection_service = tonic_reflection::server::Builder::configure().register_encoded_file_descriptor_set(gs_service::api::proto::am::gs::v1::FILE_DESCRIPTOR_SET).build_v1()?;
    Server::builder().add_service(GeoServiceServer::new(service)).add_service(reflection_service).serve(addr).await?;
    Ok(())
}
