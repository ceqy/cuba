use pp_service::api::grpc_server::PpServiceImpl;
use pp_service::api::proto::mf::pp::v1::production_planning_service_server::ProductionPlanningServiceServer;
use pp_service::application::handlers::RunMrpHandler;
use pp_service::infrastructure::repository::PlannedOrderRepository;
use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50058).await?;
    let pool = context.db_pool;
    let addr = context.addr;

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
        .register_encoded_file_descriptor_set(
            pp_service::api::proto::mf::pp::v1::FILE_DESCRIPTOR_SET,
        )
        .build_v1()?;

    info!("MF Production Planning Service listening on {}", addr);

    Server::builder()
        .add_service(ProductionPlanningServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
