use tonic::transport::Server;
use iam_service::infrastructure::grpc::iam::v1::auth_service_server::AuthServiceServer;
use iam_service::api::grpc_server::AuthServiceImpl;
use iam_service::infrastructure::persistence::postgres_user_repository::PostgresUserRepository;
use iam_service::infrastructure::bcrypt_password_service::BcryptPasswordService;
use iam_service::application::handlers::RegisterUserHandler;
use iam_service::application::handlers::LoginUserHandler;
use iam_service::infrastructure::jwt_token_service::JwtTokenService;
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

    // Run migrations
    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    // Infrastructure
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let password_service = Arc::new(BcryptPasswordService::default());
    let token_service = Arc::new(JwtTokenService::new("super_secret_key".to_string())); // TODO: Load from env
    
    // Application
    let register_handler = Arc::new(RegisterUserHandler::new(user_repo.clone(), password_service.clone()));
    let login_handler = Arc::new(LoginUserHandler::new(user_repo.clone(), password_service.clone(), token_service));
    
    // API
    let auth_service = AuthServiceImpl::new(register_handler, login_handler);
    
    // Reflection Service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(iam_service::infrastructure::grpc::iam::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("Server listening on {}", addr);
    
    Server::builder()
        .add_service(AuthServiceServer::new(auth_service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
