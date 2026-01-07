use tonic::transport::Channel;
use crate::dto::auth::{
    LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, 
    RefreshTokenRequest, RefreshTokenResponse, UserInfo
};
use anyhow::Result;

// 引入编译生成的 proto 模块
pub mod pb {
    tonic::include_proto!("auth");
}

use pb::auth_service_client::AuthServiceClient;

#[derive(Clone)]
pub struct AuthClient {
    // 使用 Tonic 的 Channel，它是轻量且可克隆的
    inner: AuthServiceClient<Channel>,
}

impl AuthClient {
    pub async fn connect(url: String) -> Result<Self> {
        let inner = AuthServiceClient::connect(url).await?;
        Ok(Self { inner })
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<RegisterResponse> {
        let mut client = self.inner.clone();
        
        // DTO -> Proto
        let request = tonic::Request::new(pb::RegisterRequest {
            username: req.username,
            email: req.email,
            password: req.password,
            tenant_id: if req.tenant_id.is_empty() { "default".to_string() } else { req.tenant_id },
        });

        // RPC Call
        let response = client.register(request).await?.into_inner();

        // Proto -> DTO
        let user = response.user.unwrap_or_default();
        Ok(RegisterResponse {
            user_id: user.user_id,
            username: user.username,
            email: user.email,
        })
    }

    pub async fn login(&self, req: LoginRequest) -> Result<LoginResponse> {
        let mut client = self.inner.clone();
        
        let request = tonic::Request::new(pb::LoginRequest {
            username: req.username,
            password: req.password,
            tenant_id: if req.tenant_id.is_empty() { "default".to_string() } else { req.tenant_id },
        });

        let response = client.login(request).await?.into_inner();
        let user = response.user.unwrap_or_default();

        Ok(LoginResponse {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            expires_in: response.expires_in,
            user: UserInfo {
                user_id: user.user_id,
                username: user.username,
                email: user.email,
                display_name: user.display_name,
                avatar_url: user.avatar_url,
                tenant_id: user.tenant_id,
                roles: user.roles,
            },
        })
    }

    pub async fn refresh_token(&self, req: RefreshTokenRequest) -> Result<RefreshTokenResponse> {
        let mut client = self.inner.clone();
        
        let request = tonic::Request::new(pb::RefreshTokenRequest {
            refresh_token: req.refresh_token,
        });

        let response = client.refresh_token(request).await?.into_inner();

        Ok(RefreshTokenResponse {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            expires_in: response.expires_in,
        })
    }

    pub async fn get_user_info(&self, token: &str) -> Result<UserInfo> {
        // 通常这里应该使用 Authorization header 传递 token，
        // 但 auth_service.proto 中 GetUserInfoRequest 定义为接收 user_id (通常) 或 token。
        // 让我们查看 Auth Service 的 UserInfo 定义：
        // rpc UserInfo(UserInfoRequest) returns (UserInfoResponse);
        // message UserInfoRequest { string access_token = 1; }
        
        // Wait, there are TWO user info RPCs in proto:
        // 1. rpc GetUserInfo(GetUserInfoRequest) returns (GetUserInfoResponse); // admin or self by user_id?
        // 2. rpc UserInfo(UserInfoRequest) returns (UserInfoResponse); // OIDC style, by token
        
        // 使用 OIDC 风格的 UserInfo 接口更适合网关透传 Token
        let mut client = self.inner.clone();
        
        let request = tonic::Request::new(pb::UserInfoRequest {
            access_token: token.to_string(),
        });

        let response = client.user_info(request).await?.into_inner();

        Ok(UserInfo {
            user_id: response.sub,
            username: response.name.clone(), // Map name to username or display_name? OIDC mappings vary.
            email: response.email,
            display_name: response.name,
            avatar_url: response.picture,
            tenant_id: "default".to_string(), // OIDC UserInfo response might not have tenant_id in top level
            roles: vec![], // OIDC standard claims don't strictly assure roles, might be in custom claims
        })
    }
}
