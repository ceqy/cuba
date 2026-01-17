use tonic::transport::Server;
use tracing::info;
use std::sync::Arc;
use auth_service::infrastructure::grpc::iam::auth::v1::auth_service_server::AuthServiceServer;
use auth_service::api::grpc_server::AuthServiceImpl;
use auth_service::infrastructure::persistence::{PostgresUserRepository, PostgresUserSessionRepository};
use auth_service::infrastructure::bcrypt_password_service::BcryptPasswordService;
use auth_service::infrastructure::jwt_token_service::JwtTokenService;
use auth_service::infrastructure::rbac_client::RbacClient;
use auth_service::application::handlers::{RegisterUserHandler, LoginUserHandler, RefreshTokenHandler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50051).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    // Infrastructure
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let session_repo = Arc::new(PostgresUserSessionRepository::new(pool.clone()));
    let password_service = Arc::new(BcryptPasswordService::default());

    // Read JWT secret from environment variable
    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable must be set");
    let token_service = Arc::new(JwtTokenService::new(jwt_secret));
    
    // RBAC Client (connecting to the service in K8s or local)
    let rbac_addr = std::env::var("RBAC_SERVICE_ADDR").unwrap_or_else(|_| "http://rbac-service.cuba-iam.svc.cluster.local:50052".to_string());
    let rbac_client = Arc::new(RbacClient::new(rbac_addr).await?);
    
    // Handlers
    let register_handler = Arc::new(RegisterUserHandler::new(user_repo.clone(), password_service.clone()));
    let login_handler = Arc::new(LoginUserHandler::new(
        user_repo.clone(), 
        session_repo.clone(),
        rbac_client.clone(),
        password_service.clone(), 
        token_service.clone()
    ));
    let refresh_token_handler = Arc::new(RefreshTokenHandler::new(
        user_repo.clone(),
        session_repo.clone(),
        rbac_client.clone(),
        token_service.clone()
    ));
    
    // Service
    let auth_service = AuthServiceImpl::new(
        register_handler, 
        login_handler, 
        refresh_token_handler,
        user_repo.clone()
    );
    
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
