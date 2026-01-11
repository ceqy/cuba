use tonic::transport::Server;
use gl_service::infrastructure::grpc::fi::gl::v1::gl_journal_entry_service_server::GlJournalEntryServiceServer;
use gl_service::api::grpc_server::GlServiceImpl;
use gl_service::infrastructure::persistence::postgres_journal_repository::PostgresJournalRepository;
use gl_service::application::handlers::{
    CreateJournalEntryHandler,
    GetJournalEntryHandler,
    ListJournalEntriesHandler,
    PostJournalEntryHandler,
};
use cuba_database::{DatabaseConfig, init_pool};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    
    let addr = "0.0.0.0:50052".parse()?;
    info!("Starting GL Service on {}", addr);

    // Database
    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

    // Run migrations
    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    // Infrastructure
    let journal_repo = Arc::new(PostgresJournalRepository::new(pool.clone()));
    
    // Application Handlers
    let create_handler = Arc::new(CreateJournalEntryHandler::new(journal_repo.clone()));
    let get_handler = Arc::new(GetJournalEntryHandler::new(journal_repo.clone()));
    let list_handler = Arc::new(ListJournalEntriesHandler::new(journal_repo.clone()));
    let post_handler = Arc::new(PostJournalEntryHandler::new(journal_repo.clone()));
    
    // API
    let gl_service = GlServiceImpl::new(
        create_handler,
        get_handler,
        list_handler,
        post_handler,
    );
    
    // Reflection Service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(gl_service::infrastructure::grpc::fi::gl::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("GL Service listening on {}", addr);
    
    Server::builder()
        .add_service(GlJournalEntryServiceServer::new(gl_service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
