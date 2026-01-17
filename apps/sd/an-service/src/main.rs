use tonic::transport::Server;
use tracing::info;
use std::sync::Arc;
use an_service::api::grpc_server::AnServiceImpl;
use an_service::api::proto::sd::an::v1::sales_analytics_service_server::SalesAnalyticsServiceServer;
use an_service::infrastructure::repository::SalesRepository;
use an_service::application::handlers::SalesHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50079).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    let repo = Arc::new(SalesRepository::new(pool.clone()));
    let handler = Arc::new(SalesHandler::new(repo));
    let service = AnServiceImpl::new(handler);
    
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(an_service::api::proto::sd::an::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("SD Sales Analytics Service listening on {}", addr);
    
    Server::builder()
        .add_service(SalesAnalyticsServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
