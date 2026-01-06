use anyhow::Result;

// Import shared libs
use cuba_config::load_config;
use cuba_telemetry::init_subscriber;

// Import service-specific modules
// mod domain;
// mod application;
// mod infrastructure;
// mod grpc;

// Placeholder for service settings
#[derive(Debug, serde::Deserialize)]
struct AuthServiceSettings {
    server_addr: String,
    // db_url: String,
    // kafka_brokers: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize telemetry
    init_subscriber("auth-service");

    // 2. Load configuration
    let settings: AuthServiceSettings = load_config("auth-service")?;
    tracing::info!("Configuration loaded: {:?}", settings);

    // 3. Build DI Container
    // let container = build_container(&settings).await?;

    // 4. Setup gRPC Server
    // let auth_grpc_service = grpc::AuthServiceImpl::new(container.command_bus, container.query_bus);

    let addr: std::net::SocketAddr = settings.server_addr.parse()?;
    tracing::info!("Auth service listening on {}", addr);

    // Server::builder()
    //     .add_service(AuthServiceServer::new(auth_grpc_service))
    //     .serve(addr)
    //     .await?;

    println!("Server would start at {}. Press Ctrl+C to exit.", addr);
    tokio::signal::ctrl_c().await?;

    Ok(())
}

// Placeholder for DI container
// async fn build_container(settings: &AuthServiceSettings) -> Result<DIContainer> {
//     ...
// }
