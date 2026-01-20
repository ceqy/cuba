use qi_service::api::grpc_server::QiServiceImpl;
use qi_service::api::proto::mf::qi::v1::quality_inspection_service_server::QualityInspectionServiceServer;
use qi_service::application::handlers::InspectionHandler;
use qi_service::infrastructure::repository::InspectionLotRepository;
use std::sync::Arc;
use tonic::transport::Server;
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

    // Infrastructure
    let repo = Arc::new(InspectionLotRepository::new(pool.clone()));

    // Application Handlers
    let handler = Arc::new(InspectionHandler::new(repo.clone()));

    // API
    let service = QiServiceImpl::new(handler, repo);

    // Reflection Service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(
            qi_service::api::proto::mf::qi::v1::FILE_DESCRIPTOR_SET,
        )
        .build_v1()?;

    info!("MF Quality Inspection Service listening on {}", addr);

    Server::builder()
        .add_service(QualityInspectionServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
