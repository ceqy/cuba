use oauth_service::api::grpc_server::OAuthServiceImpl;
use oauth_service::application::handlers::{AuthorizeHandler, TokenHandler};
use oauth_service::infrastructure::grpc::iam::oauth::v1::o_auth_service_server::OAuthServiceServer;
use oauth_service::infrastructure::persistence::PostgresOAuthRepository;
use oauth_service::infrastructure::services::{ClientSecretService, CryptoService, JwtService};
use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50053).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    // Infrastructure
    let oauth_repo = Arc::new(PostgresOAuthRepository::new(pool.clone()));

    // Read JWT secret from environment variable (no default fallback for security)
    let jwt_secret =
        std::env::var("JWT_SECRET").expect("JWT_SECRET environment variable must be set");
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
