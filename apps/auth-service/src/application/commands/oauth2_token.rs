//! OAuth2 Token Handler

use crate::domain::repositories::{VerificationRepository, VerificationTokenType, UserRepository, RepositoryError};
use crate::domain::services::{TokenService, TokenError};
use std::sync::Arc;

pub struct OAuth2TokenCommand {
    pub grant_type: String,
    pub code: Option<String>,
    pub refresh_token: Option<String>,
    pub client_id: String,
    pub redirect_uri: String,
    pub tenant_id: String,
}

pub struct OAuth2TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

pub struct OAuth2TokenHandler {
    verification_repo: Arc<dyn VerificationRepository>,
    user_repo: Arc<dyn UserRepository>,
    token_service: Arc<dyn TokenService>,
}

impl OAuth2TokenHandler {
    pub fn new(
        verification_repo: Arc<dyn VerificationRepository>,
        user_repo: Arc<dyn UserRepository>,
        token_service: Arc<dyn TokenService>,
    ) -> Self {
        Self {
            verification_repo,
            user_repo,
            token_service,
        }
    }

    pub async fn handle(&self, command: OAuth2TokenCommand) -> Result<OAuth2TokenResponse, String> {
        match command.grant_type.as_str() {
            "authorization_code" => self.handle_authorization_code(command).await,
            "refresh_token" => self.handle_refresh_token(command).await,
            _ => Err("Unsupported grant type".to_string()),
        }
    }

    async fn handle_authorization_code(&self, command: OAuth2TokenCommand) -> Result<OAuth2TokenResponse, String> {
        let code = command.code.ok_or("Code is required")?;
        
        // 1. 验证授权码
        let token_data = self.verification_repo.find_by_hash(&code, VerificationTokenType::OAuth2Code)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Invalid or expired code")?;

        if token_data.used_at.is_some() {
            return Err("Code already used".to_string());
        }

        if token_data.expires_at < chrono::Utc::now() {
            return Err("Code expired".to_string());
        }

        // 2. 标记为已使用
        self.verification_repo.mark_as_used(&code).await.map_err(|e| e.to_string())?;

        // 3. 获取用户并生成 Token
        let user = self.user_repo.find_by_id(&token_data.user_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("User not found")?;

        let permissions = self.user_repo.get_user_permissions(user.id())
            .await
            .map_err(|e| e.to_string())?;

        let roles_str: Vec<String> = user.roles().iter().map(|r| r.to_string()).collect();
        let perms_str: Vec<String> = permissions.into_iter().map(|p| p.to_string()).collect();

        let (access, refresh, expires) = self.token_service.generate_tokens(
            user.id().to_string(),
            command.tenant_id,
            roles_str,
            perms_str,
        ).await.map_err(|e| e.to_string())?;

        Ok(OAuth2TokenResponse {
            access_token: access,
            refresh_token: refresh,
            expires_in: expires,
        })
    }

    async fn handle_refresh_token(&self, command: OAuth2TokenCommand) -> Result<OAuth2TokenResponse, String> {
        let refresh_token = command.refresh_token.ok_or("Refresh token is required")?;
        
        let (access, refresh, expires) = self.token_service.refresh_tokens(&refresh_token)
            .await
            .map_err(|e| e.to_string())?;

        Ok(OAuth2TokenResponse {
            access_token: access,
            refresh_token: refresh,
            expires_in: expires,
        })
    }
}
