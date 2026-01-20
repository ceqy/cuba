use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50062).await?;
    let _pool = context.db_pool;
    let addr = context.addr;

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
