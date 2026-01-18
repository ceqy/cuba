use tonic::transport::Server;
use tracing::info;

use ar_service::api::proto::fi::ap::v1::accounts_receivable_payable_service_server::AccountsReceivablePayableServiceServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50062).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    // Run migrations
    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;

    // Initialize GL Client
    let gl_client = cuba_finance::create_gl_client(
        "http://gl-service.cuba-fi.svc.cluster.local:50060"
    ).await?;

    // Create AR Service with all dependencies wired up
    let ar_service = ar_service::create_ar_service(pool, gl_client);

    // Reflection Service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(ar_service::api::proto::fi::ap::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("AR Service listening on {}", addr);

    Server::builder()
        .add_service(AccountsReceivablePayableServiceServer::new(ar_service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
