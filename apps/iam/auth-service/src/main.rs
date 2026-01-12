use tonic::transport::Server;
use cuba_database::{DatabaseConfig, init_pool};
use tracing::info;
use std::sync::Arc;
use auth_service::infrastructure::grpc::iam::auth::v1::auth_service_server::AuthServiceServer;
use auth_service::api::grpc_server::AuthServiceImpl;
use auth_service::infrastructure::persistence::postgres_user_repository::PostgresUserRepository;
use auth_service::infrastructure::bcrypt_password_service::BcryptPasswordService;
use auth_service::infrastructure::jwt_token_service::JwtTokenService;
use auth_service::application::handlers::{RegisterUserHandler, LoginUserHandler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    
    let addr = "0.0.0.0:50051".parse()?;
    info!("Starting auth-service on {}", addr);

    // Database
    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

    // Infrastructure
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let password_service = Arc::new(BcryptPasswordService::default());
    let token_service = Arc::new(JwtTokenService::new("super_secret_key".to_string())); // TODO: Env
    
    // Handlers
    let register_handler = Arc::new(RegisterUserHandler::new(user_repo.clone(), password_service.clone()));
    let login_handler = Arc::new(LoginUserHandler::new(user_repo.clone(), password_service.clone(), token_service.clone()));
    
    // Service
    let auth_service = AuthServiceImpl::new(register_handler, login_handler, user_repo.clone());
    
    // Reflection
    let descriptor = include_bytes!(concat!(env!("OUT_DIR"), "/descriptor.bin"));
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(descriptor)
        .build_v1()?;

    info!("Server listening on {}", addr);
    
    Server::builder()
        .add_service(reflection_service)
        .add_service(AuthServiceServer::new(auth_service))
        .serve(addr)
        .await?;

    Ok(())
}
