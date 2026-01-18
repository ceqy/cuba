use tonic::transport::Server;
use tracing::info;
use std::sync::Arc;

use co_service::api::grpc_server::CoServiceImpl;
use co_service::api::proto::fi::co::v1::controlling_allocation_service_server::ControllingAllocationServiceServer;
use co_service::infrastructure::repository::AllocationRepository;
use co_service::application::handlers::AllocationHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50063).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;

    // Initialize GL Client using unified function
    let gl_client = cuba_finance::create_gl_client(
        "http://gl-service.cuba-fi.svc.cluster.local:50060"
    ).await?;
    
    let repo = Arc::new(AllocationRepository::new(pool.clone()));
    let handler = Arc::new(AllocationHandler::new(repo.clone(), gl_client));
    let service = CoServiceImpl::new(handler, repo);
    
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(co_service::api::proto::fi::co::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("FI Controlling Allocation Service listening on {}", addr);
    
    Server::builder()
        .add_service(ControllingAllocationServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
