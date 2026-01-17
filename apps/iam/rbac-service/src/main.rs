use tonic::transport::Server;
use tracing::info;
use std::sync::Arc;
use rbac_service::infrastructure::grpc::iam::rbac::v1::rbac_service_server::RbacServiceServer;
use rbac_service::api::grpc_server::RBACServiceImpl;
use rbac_service::infrastructure::PostgresRbacRepository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50052).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    // Infrastructure
    let rbac_repo = Arc::new(PostgresRbacRepository::new(pool.clone()));
    
    // Service
    let rbac_service = RBACServiceImpl::new(rbac_repo.clone(), rbac_repo.clone());
    
    // Reflection
    let descriptor = include_bytes!(concat!(env!("OUT_DIR"), "/descriptor.bin"));
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(descriptor)
        .build_v1()?;

    info!("Server listening on {}", addr);
    
    Server::builder()
        .add_service(reflection_service)
        .add_service(RbacServiceServer::new(rbac_service))
        .serve(addr)
        .await?;

    Ok(())
}
