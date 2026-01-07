//! Social Login Command Handler

use crate::application::dto::{LoginResponseDto, UserDto};
use crate::domain::repositories::UserRepository;
use crate::domain::services::{TokenService, TokenError};
use crate::domain::aggregates::User;
use crate::infrastructure::services::social_auth::SocialAuthService;
use crate::domain::DomainError;
use std::sync::Arc;

pub struct SocialLoginCommand {
    pub provider: String,
    pub code: String,
    pub redirect_uri: String,
}

pub struct SocialLoginHandler {
    user_repo: Arc<dyn UserRepository>,
    token_service: Arc<dyn TokenService>,
    social_auth_service: Arc<SocialAuthService>,
}

impl SocialLoginHandler {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        token_service: Arc<dyn TokenService>,
        social_auth_service: Arc<SocialAuthService>,
    ) -> Self {
        Self {
            user_repo,
            token_service,
            social_auth_service,
        }
    }

    pub async fn handle(&self, command: SocialLoginCommand) -> Result<LoginResponseDto, DomainError> {
        // 1. 获取对应的 Provider
        let provider = self.social_auth_service.get_provider(&command.provider)?;

        // 2. 验证 Code 并获取用户信息
        let profile = provider.verify_code(&command.code, &command.redirect_uri).await?;

        // 3. 查找或创建用户
        // 假设 email 是唯一的且受信的 (Social Login 通常要求 email verified)
        let user = match self.user_repo.find_by_email(&profile.email).await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))? 
        {
            Some(existing_user) => existing_user,
            None => {
                // 自动注册新用户
                // 生成随机密码 (因为是 social login，用户不需要知道)
                let random_password = uuid::Uuid::new_v4().to_string();
                let mut new_user = User::register(
                    profile.email.clone(), // 使用 email 作为 username
                    profile.email.clone(),
                    &random_password,
                ).map_err(|e| DomainError::InvalidInput(e.to_string()))?;
                
                // 可以在这里更新头像等详细信息
                // new_user.update_profile(...)

                self.user_repo.save(&mut new_user).await
                    .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
                
                new_user
            }
        };

        // 4. 生成 Token
        // TODO: 获取用户实际的 role names and permission strings
        let roles: Vec<String> = vec![]; // user.roles...
        let permissions: Vec<String> = vec![]; 

        let (access_token, refresh_token, expires_in) = self.token_service.generate_tokens(
            user.id().to_string(),
            String::new(), // Default/Empty Tenant ID for now
            roles,
            permissions
        ).await
            .map_err(|_| DomainError::InternalError("Failed to generate tokens".to_string()))?;

        Ok(LoginResponseDto {
            access_token: Some(access_token),
            refresh_token: Some(refresh_token),
            expires_in: Some(expires_in),
            user: UserDto::from_user(&user),
            requires_2fa: false, // Social Login 通常跳过 2FA，或者后续再加强
            temp_token: None,
        })
    }
}
