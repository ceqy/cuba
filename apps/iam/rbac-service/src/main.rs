use tonic::transport::Server;
use cuba_database::{DatabaseConfig, init_pool};
use tracing::info;
use std::sync::Arc;
use rbac_service::infrastructure::grpc::iam::rbac::v1::rbac_service_server::RbacServiceServer;
use rbac_service::api::grpc_server::RBACServiceImpl;
use rbac_service::infrastructure::PostgresRbacRepository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    
    let addr = "0.0.0.0:50052".parse()?;
    info!("Starting rbac-service on {}", addr);

    // Database
    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

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
