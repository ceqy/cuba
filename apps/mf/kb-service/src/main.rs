use tonic::transport::Server;
use tracing::info;
use dotenvy::dotenv;
use std::sync::Arc;
use cuba_database::{DatabaseConfig, init_pool};

use kb_service::api::grpc_server::KbServiceImpl;
use kb_service::api::proto::mf::kb::v1::kanban_service_server::KanbanServiceServer;
use kb_service::infrastructure::repository::KanbanRepository;
use kb_service::application::handlers::KanbanHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    dotenv().ok();
    
    let addr = "0.0.0.0:50074".parse()?;
    info!("Starting MF Kanban Service on {}", addr);

    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    let repo = Arc::new(KanbanRepository::new(pool.clone()));
    let handler = Arc::new(KanbanHandler::new(repo.clone()));
    let service = KbServiceImpl::new(handler, repo);
    
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(kb_service::api::proto::mf::kb::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("MF Kanban Service listening on {}", addr);
    
    Server::builder()
        .add_service(KanbanServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
