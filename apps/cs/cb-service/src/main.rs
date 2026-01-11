use tonic::transport::Server;
use cuba_database::{DatabaseConfig, init_pool};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Init Telemetry
    cuba_telemetry::init_telemetry();
    
    // 2. Load Config
    // In a real app we might load strictly typed config, here we assume env vars.
    let addr = "0.0.0.0:50082".parse()?;
    info!("Starting cb-service on {}", addr);

    // 3. Init Database
    let db_config = DatabaseConfig::default();
    let _pool = init_pool(&db_config).await?; // Pool is ready, typically passed to repositories

    // 4. Init Reflection
    let descriptor = include_bytes!(concat!(env!("OUT_DIR"), "/descriptor.bin"));
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(descriptor)
        .build_v1()?;

    info!("Service listening on {}", addr);
    
    // 5. Start Server
    Server::builder()
        .add_service(reflection_service)
        // .add_service(YourGrpcServiceServer::new(YourServiceImpl))
        .serve(addr)
        .await?;

    Ok(())
}
