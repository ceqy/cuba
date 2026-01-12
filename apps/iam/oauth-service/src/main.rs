use tonic::transport::Server;
use tracing::info;
use oauth_service::infrastructure::grpc::iam::oauth::v1::o_auth_service_server::OAuthServiceServer;
use oauth_service::api::grpc_server::OAuthServiceImpl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    
    let addr = "0.0.0.0:50053".parse()?;
    info!("Starting oauth-service on {}", addr);
    
    // Service
    let oauth_service = OAuthServiceImpl::new();
    
    // Reflection
    let descriptor = include_bytes!(concat!(env!("OUT_DIR"), "/descriptor.bin"));
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(descriptor)
        .build_v1()?;

    info!("Server listening on {}", addr);
    
    Server::builder()
        .add_service(reflection_service)
        .add_service(OAuthServiceServer::new(oauth_service))
        .serve(addr)
        .await?;

    Ok(())
}
