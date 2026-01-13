use tonic::transport::Server;
use cuba_database::{DatabaseConfig, init_pool};
use tracing::info;
use std::sync::Arc;
use oauth_service::infrastructure::grpc::iam::oauth::v1::o_auth_service_server::OAuthServiceServer;
use oauth_service::api::grpc_server::OAuthServiceImpl;
use oauth_service::infrastructure::persistence::PostgresOAuthRepository;
use oauth_service::infrastructure::services::{JwtService, CryptoService, ClientSecretService};
use oauth_service::application::handlers::{AuthorizeHandler, TokenHandler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cuba_telemetry::init_telemetry();
    
    let addr = "0.0.0.0:50053".parse()?;
    info!("Starting oauth-service on {}", addr);

    // Database
    let db_config = DatabaseConfig::default();
    let pool = init_pool(&db_config).await?;

    // Infrastructure
    let oauth_repo = Arc::new(PostgresOAuthRepository::new(pool.clone()));
    
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "cuba-secret-key-change-me".to_string());
    let jwt_service = Arc::new(JwtService::new(jwt_secret));
    let crypto_service = Arc::new(CryptoService::default());
    let secret_service = Arc::new(ClientSecretService::default());

    // Application Handlers
    let authorize_handler = Arc::new(AuthorizeHandler::new(
        oauth_repo.clone(),
        oauth_repo.clone(),
        crypto_service.clone(),
    ));

    let token_handler = Arc::new(TokenHandler::new(
        oauth_repo.clone(),
        oauth_repo.clone(),
        oauth_repo.clone(),
        jwt_service.clone(),
        crypto_service.clone(),
        secret_service.clone(),
    ));
    
    // Service
    let oauth_service = OAuthServiceImpl::new(
        authorize_handler,
        token_handler,
        oauth_repo.clone(),
        oauth_repo.clone(), // refresh_token_repo
        secret_service,
        jwt_service,
    );
    
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
