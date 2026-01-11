use tonic::transport::Server;
use tracing::info;
use dotenvy::dotenv;
use std::sync::Arc;
use cuba_database::{DatabaseConfig, init_pool};
use rust_decimal::Decimal;

use sf_service::api::grpc_server::SfServiceImpl;
use sf_service::api::proto::mf::sf::v1::shop_floor_execution_service_server::ShopFloorExecutionServiceServer;
use sf_service::infrastructure::repository::ProductionOrderRepository;
use sf_service::application::handlers::ProductionHandler;
use sf_service::application::commands::CreateProductionOrderCommand;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    dotenv().ok();
    
    // Port 50059 (PP 58, SF 59)
    let addr = "0.0.0.0:50059".parse()?;
    info!("Starting MF Shop Floor Service on {}", addr);

    // Database
    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

    // Run migrations
    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    // Infrastructure
    let repo = Arc::new(ProductionOrderRepository::new(pool.clone()));
    
    // Application Handlers
    let handler = Arc::new(ProductionHandler::new(repo.clone()));
    
    // SEED DATA HACK for Verification
    // In a real app, this wouldn't be here. 
    // We check if DB is emptyish or just blindly insert one specific order for "curl" testing availability.
    if std::env::var("SEED_DATA").unwrap_or_default() == "true" {
        info!("Seeding Production Order...");
        let _ = handler.create_seed_order(CreateProductionOrderCommand {
            material: "MAT001".to_string(),
            plant: "1000".to_string(),
            quantity: Decimal::from(100),
            unit: "PC".to_string(),
        }).await;
    }

    // API
    let service = SfServiceImpl::new(handler, repo);
    
    // Reflection Service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(sf_service::api::proto::mf::sf::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("MF Shop Floor Service listening on {}", addr);
    
    Server::builder()
        .add_service(ShopFloorExecutionServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
