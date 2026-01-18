use tonic::transport::Server;
use gl_service::infrastructure::grpc::fi::gl::v1::gl_journal_entry_service_server::GlJournalEntryServiceServer;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50060).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    // Run migrations
    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;

    // Get COA service endpoint (optional)
    let coa_endpoint = std::env::var("COA_SERVICE_URL")
        .ok()
        .or_else(|| Some("http://coa-service.cuba-fi.svc.cluster.local:50065".to_string()));

    // Create GL Service with all dependencies wired up
    let gl_service = gl_service::create_gl_service(pool, coa_endpoint).await?;

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
