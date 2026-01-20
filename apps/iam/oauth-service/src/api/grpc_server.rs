use crate::application::handlers::{AuthorizeHandler, TokenHandler};
use crate::domain::repositories::{AuthCodeRepository, ClientRepository, RefreshTokenRepository};
use crate::infrastructure::grpc::iam::oauth::v1::o_auth_service_server::OAuthService;
use crate::infrastructure::grpc::iam::oauth::v1::*;
use crate::infrastructure::services::{ClientSecretService, JwtService};
use chrono::Utc;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tonic::{Request, Response, Status};

pub struct OAuthServiceImpl<C, A, R>
where
    C: ClientRepository + 'static,
    A: AuthCodeRepository + 'static,
    R: RefreshTokenRepository + 'static,
{
    authorize_handler: Arc<AuthorizeHandler<C, A>>,
    token_handler: Arc<TokenHandler<C, A, R>>,
    client_repo: Arc<C>,
    refresh_token_repo: Arc<R>,
    secret_service: Arc<ClientSecretService>,
    jwt_service: Arc<JwtService>,
}

impl<C, A, R> OAuthServiceImpl<C, A, R>
where
    C: ClientRepository + 'static,
    A: AuthCodeRepository + 'static,
    R: RefreshTokenRepository + 'static,
{
    pub fn new(
        authorize_handler: Arc<AuthorizeHandler<C, A>>,
        token_handler: Arc<TokenHandler<C, A, R>>,
        client_repo: Arc<C>,
        refresh_token_repo: Arc<R>,
        secret_service: Arc<ClientSecretService>,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            authorize_handler,
            token_handler,
            client_repo,
            refresh_token_repo,
            secret_service,
            jwt_service,
        }
    }
}

#[tonic::async_trait]
impl<C, A, R> OAuthService for OAuthServiceImpl<C, A, R>
where
    C: ClientRepository + 'static,
    A: AuthCodeRepository + 'static,
    R: RefreshTokenRepository + 'static,
{
    async fn authorize(
        &self,
        request: Request<AuthorizeRequest>,
    ) -> Result<Response<AuthorizeResponse>, Status> {
        // Get metadata before moving request
        let user_id = request
            .metadata()
            .get("x-user-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .ok_or_else(|| Status::unauthenticated("User ID not found in context"))?;

        let req = request.into_inner();

        let code = self
            .authorize_handler
            .handle(
                req.client_id,
                user_id,
                req.redirect_uri,
                req.scope,
                if req.code_challenge.is_empty() {
                    None
                } else {
                    Some(req.code_challenge)
                },
                None, // Default to plain if not specified for PKCE for now
            )
            .await
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        Ok(Response::new(AuthorizeResponse {
            code,
            state: req.state,
        }))
    }

    async fn token(
        &self,
        request: Request<TokenRequest>,
    ) -> Result<Response<TokenResponse>, Status> {
        let req = request.into_inner();

        match req.grant_type.as_str() {
            "authorization_code" => {
                let (access_token, refresh_token, expires_in) = self
                    .token_handler
                    .handle_auth_code(
                        req.code,
                        req.client_id,
                        req.client_secret,
                        req.redirect_uri,
                        if req.code_verifier.is_empty() {
                            None
                        } else {
                            Some(req.code_verifier)
                        },
                    )
                    .await
                    .map_err(|e| Status::unauthenticated(e.to_string()))?;

                Ok(Response::new(TokenResponse {
                    access_token,
                    token_type: "Bearer".to_string(),
                    expires_in,
                    refresh_token,
                    scope: "".to_string(),
                }))
            },
            "client_credentials" => {
                let (access_token, expires_in) = self
                    .token_handler
                    .handle_client_credentials(
                        req.client_id,
                        req.client_secret,
                        "".to_string(), // Scope hint
                    )
                    .await
                    .map_err(|e| Status::unauthenticated(e.to_string()))?;

                Ok(Response::new(TokenResponse {
                    access_token,
                    token_type: "Bearer".to_string(),
                    expires_in,
                    refresh_token: "".to_string(),
                    scope: "".to_string(),
                }))
            },
            "refresh_token" => {
                let (access_token, refresh_token, expires_in) = self
                    .token_handler
                    .handle_refresh_token(req.refresh_token, req.client_id, req.client_secret)
                    .await
                    .map_err(|e| Status::unauthenticated(e.to_string()))?;

                Ok(Response::new(TokenResponse {
                    access_token,
                    token_type: "Bearer".to_string(),
                    expires_in,
                    refresh_token,
                    scope: "".to_string(),
                }))
            },
            _ => Err(Status::invalid_argument("Unsupported grant type")),
        }
    }

    async fn revoke_token(
        &self,
        request: Request<RevokeTokenRequest>,
    ) -> Result<Response<RevokeTokenResponse>, Status> {
        let req = request.into_inner();

        // Check token type hint to decide which to revoke
        match req.token_type_hint.as_str() {
            "refresh_token" | "" => {
                // Try to delete as refresh token
                self.refresh_token_repo
                    .delete(&req.token)
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?;
            },
            "access_token" => {
                // Access tokens are stateless JWTs, we can't revoke them directly
                // In a real implementation, you'd add them to a blocklist
                // For now, we just return success
            },
            _ => return Err(Status::invalid_argument("Unknown token type hint")),
        }

        Ok(Response::new(RevokeTokenResponse { success: true }))
    }

    async fn user_info(
        &self,
        request: Request<UserInfoRequest>,
    ) -> Result<Response<UserInfoResponse>, Status> {
        let req = request.into_inner();

        // Decode the access token
        let claims = self
            .jwt_service
            .decode_token(&req.access_token)
            .map_err(|e| Status::unauthenticated(format!("Invalid token: {}", e)))?;

        // Return user info from token claims
        Ok(Response::new(UserInfoResponse {
            sub: claims.sub,
            name: "".to_string(),  // Would need to fetch from user service
            email: "".to_string(), // Would need to fetch from user service
            tenant_id: claims.tenant_id,
            picture: "".to_string(),
        }))
    }

    async fn introspect_token(
        &self,
        request: Request<IntrospectTokenRequest>,
    ) -> Result<Response<IntrospectTokenResponse>, Status> {
        let req = request.into_inner();

        // Try to decode token (even if expired for introspection)
        match self.jwt_service.decode_token_unverified(&req.token) {
            Ok(claims) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);

                let is_active = (claims.exp as i64) > now;

                Ok(Response::new(IntrospectTokenResponse {
                    active: is_active,
                    scope: claims.scope,
                    client_id: claims.client_id,
                    username: "".to_string(), // Would need user lookup
                    exp: claims.exp as i64,
                    iat: claims.iat.unwrap_or(0) as i64,
                    nbf: 0,
                    sub: claims.sub,
                    aud: "".to_string(),
                    iss: "cuba-oauth".to_string(),
                    jti: "".to_string(),
                }))
            },
            Err(_) => {
                // Token is invalid
                Ok(Response::new(IntrospectTokenResponse {
                    active: false,
                    scope: "".to_string(),
                    client_id: "".to_string(),
                    username: "".to_string(),
                    exp: 0,
                    iat: 0,
                    nbf: 0,
                    sub: "".to_string(),
                    aud: "".to_string(),
                    iss: "".to_string(),
                    jti: "".to_string(),
                }))
            },
        }
    }

    async fn create_client(
        &self,
        request: Request<CreateClientRequest>,
    ) -> Result<Response<OAuthClient>, Status> {
        let req = request.into_inner();

        // Generate client secret
        let raw_secret = uuid::Uuid::new_v4().to_string().replace("-", "");
        let hashed_secret = self
            .secret_service
            .hash_secret(&raw_secret)
            .map_err(|e| Status::internal(e.to_string()))?;

        let client = crate::domain::entities::OAuthClient {
            client_id: uuid::Uuid::new_v4().to_string(),
            client_secret: hashed_secret,
            name: req.name,
            redirect_uris: req.redirect_uris,
            grant_types: req.grant_types,
            scopes: req.scopes,
            created_at: Utc::now(),
        };

        self.client_repo
            .save(&client)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(OAuthClient {
            client_id: client.client_id,
            client_secret: raw_secret, // Return raw secret ONLY on creation
            name: client.name,
            redirect_uris: client.redirect_uris,
            grant_types: client.grant_types,
            scopes: client.scopes,
            created_at: Some(prost_types::Timestamp::from(std::time::SystemTime::from(
                client.created_at,
            ))),
        }))
    }

    async fn list_clients(
        &self,
        _request: Request<ListClientsRequest>,
    ) -> Result<Response<ListClientsResponse>, Status> {
        let clients = self
            .client_repo
            .list_all()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListClientsResponse {
            clients: clients
                .into_iter()
                .map(|c| OAuthClient {
                    client_id: c.client_id,
                    client_secret: "********".to_string(),
                    name: c.name,
                    redirect_uris: c.redirect_uris,
                    grant_types: c.grant_types,
                    scopes: c.scopes,
                    created_at: Some(prost_types::Timestamp::from(std::time::SystemTime::from(
                        c.created_at,
                    ))),
                })
                .collect(),
            pagination: None,
        }))
    }

    async fn delete_client(
        &self,
        request: Request<DeleteClientRequest>,
    ) -> Result<Response<DeleteClientResponse>, Status> {
        let req = request.into_inner();
        self.client_repo
            .delete(&req.client_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(DeleteClientResponse { success: true }))
    }
}
