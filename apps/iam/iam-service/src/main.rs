use tonic::transport::Server;
use iam_service::infrastructure::grpc::iam::v1::auth_service_server::AuthServiceServer;
use iam_service::api::grpc_server::AuthServiceImpl;
use iam_service::infrastructure::persistence::postgres_user_repository::PostgresUserRepository;
use iam_service::application::handlers::RegisterUserHandler;
use cuba_database::{DatabaseConfig, init_pool};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    
    let addr = "0.0.0.0:50051".parse()?;
    info!("Starting IAM Service on {}", addr);

    // Database
    let db_config = DatabaseConfig::default();
    // Use ? operator carefully, might fail if DB not running. 
    // For now, let's allow it to fail to signal requirement.
    let pool = init_pool(&db_config).await?;
    
    // Infrastructure
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    
    // Application
    let register_handler = Arc::new(RegisterUserHandler::new(user_repo));
    
    // API
    let auth_service = AuthServiceImpl::new(register_handler);
    
    info!("Server listening on {}", addr);
    
    Server::builder()
        .add_service(AuthServiceServer::new(auth_service))
        .serve(addr)
        .await?;

    Ok(())
}
