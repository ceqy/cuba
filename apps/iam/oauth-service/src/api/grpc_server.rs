use tonic::{Request, Response, Status};
use crate::infrastructure::grpc::iam::oauth::v1::o_auth_service_server::OAuthService;
use crate::infrastructure::grpc::iam::oauth::v1::*;

pub struct OAuthServiceImpl;

impl OAuthServiceImpl {
    pub fn new() -> Self {
        Self
    }
}

#[tonic::async_trait]
impl OAuthService for OAuthServiceImpl {
    async fn authorize(&self, _request: Request<AuthorizeRequest>) -> Result<Response<AuthorizeResponse>, Status> {
        Err(Status::unimplemented("OAuth authorize not implemented"))
    }

    async fn token(&self, _request: Request<TokenRequest>) -> Result<Response<TokenResponse>, Status> {
        Err(Status::unimplemented("OAuth token not implemented"))
    }

    async fn revoke_token(&self, _request: Request<RevokeTokenRequest>) -> Result<Response<RevokeTokenResponse>, Status> {
        Err(Status::unimplemented("OAuth revoke_token not implemented"))
    }

    async fn user_info(&self, _request: Request<UserInfoRequest>) -> Result<Response<UserInfoResponse>, Status> {
        Err(Status::unimplemented("OAuth user_info not implemented"))
    }

    async fn introspect_token(&self, _request: Request<IntrospectTokenRequest>) -> Result<Response<IntrospectTokenResponse>, Status> {
        Err(Status::unimplemented("OAuth introspect_token not implemented"))
    }

    async fn create_client(&self, _request: Request<CreateClientRequest>) -> Result<Response<OAuthClient>, Status> {
        Err(Status::unimplemented("OAuth create_client not implemented"))
    }

    async fn list_clients(&self, _request: Request<ListClientsRequest>) -> Result<Response<ListClientsResponse>, Status> {
        Err(Status::unimplemented("OAuth list_clients not implemented"))
    }

    async fn delete_client(&self, _request: Request<DeleteClientRequest>) -> Result<Response<DeleteClientResponse>, Status> {
        Err(Status::unimplemented("OAuth delete_client not implemented"))
    }
}
