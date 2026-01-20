use po_service::api::grpc_server::PoServiceImpl;
use po_service::api::proto::pm::po::v1::purchase_order_service_server::PurchaseOrderServiceServer;
use po_service::application::handlers::CreatePurchaseOrderHandler;
use po_service::infrastructure::repository::PurchaseOrderRepository;
use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50057).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    // Run migrations
    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;

    // Infrastructure
    let repo = Arc::new(PurchaseOrderRepository::new(pool.clone()));

    // Application Handlers
    let create_handler = Arc::new(CreatePurchaseOrderHandler::new(repo.clone()));

    // API
    let service = PoServiceImpl::new(create_handler);

    // Reflection Service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(
            po_service::api::proto::pm::po::v1::FILE_DESCRIPTOR_SET,
        )
        .build_v1()?;

    info!("PM Purchase Order Service listening on {}", addr);

    Server::builder()
        .add_service(PurchaseOrderServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
