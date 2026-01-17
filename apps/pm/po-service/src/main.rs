use tonic::transport::Server;
use tracing::info;
use std::sync::Arc;
use po_service::api::grpc_server::PoServiceImpl;
use po_service::api::proto::pm::po::v1::purchase_order_service_server::PurchaseOrderServiceServer;
use po_service::infrastructure::repository::PurchaseOrderRepository;
use po_service::application::handlers::CreatePurchaseOrderHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    dotenv().ok();
    
    // Port 50057
    let addr = "0.0.0.0:50057".parse()?;
    info!("Starting PM Purchase Order Service on {}", addr);

    // Database
    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

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
        .register_encoded_file_descriptor_set(po_service::api::proto::pm::po::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("PM Purchase Order Service listening on {}", addr);
    
    Server::builder()
        .add_service(PurchaseOrderServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
