use tonic::transport::Server;
use tracing::info;
use std::sync::Arc;
use pp_service::api::grpc_server::PpServiceImpl;
use pp_service::api::proto::mf::pp::v1::production_planning_service_server::ProductionPlanningServiceServer;
use pp_service::infrastructure::repository::PlannedOrderRepository;
use pp_service::application::handlers::RunMrpHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    dotenv().ok();
    
    // Port 50058 (SC IM 56, PM PO 57, so MF PP 58)
    let addr = "0.0.0.0:50058".parse()?;
    info!("Starting MF Production Planning Service on {}", addr);

    // Database
    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

    // Run migrations
    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    // Infrastructure
    let repo = Arc::new(PlannedOrderRepository::new(pool.clone()));
    
    // Application Handlers
    let mrp_handler = Arc::new(RunMrpHandler::new(repo.clone()));
    
    // API
    let service = PpServiceImpl::new(mrp_handler, repo);
    
    // Reflection Service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(pp_service::api::proto::mf::pp::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("MF Production Planning Service listening on {}", addr);
    
    Server::builder()
        .add_service(ProductionPlanningServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
