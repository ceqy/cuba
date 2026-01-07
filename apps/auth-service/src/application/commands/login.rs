//! Login Command
//!
//! 处理用户登录用例。

use crate::application::dto::{LoginResponseDto, UserDto};
use crate::domain::repositories::{RepositoryError, UserRepository};
use crate::domain::services::TokenService;
use std::sync::Arc;
use thiserror::Error;

/// 登录命令
#[derive(Debug, Clone)]
pub struct LoginCommand {
    pub username: String,
    pub password: String,
    pub tenant_id: String,
}

/// 登录错误
#[derive(Error, Debug)]
pub enum LoginError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Account is disabled")]
    AccountDisabled,

    #[error("Email not verified")]
    EmailNotVerified,

    #[error("Repository error: {0}")]
    RepositoryError(#[from] RepositoryError),

    #[error("Token error: {0}")]
    TokenError(String),
}

impl From<LoginError> for crate::domain::errors::DomainError {
    fn from(err: LoginError) -> Self {
        match err {
            LoginError::InvalidCredentials => crate::domain::errors::DomainError::InvalidCredentials,
            LoginError::AccountDisabled => crate::domain::errors::DomainError::AuthenticationFailed("Account disabled".to_string()),
            LoginError::EmailNotVerified => crate::domain::errors::DomainError::AuthenticationFailed("Email not verified".to_string()),
            LoginError::RepositoryError(e) => crate::domain::errors::DomainError::from(e),
            LoginError::TokenError(m) => crate::domain::errors::DomainError::InternalError(m),
        }
    }
}

/// 登录命令处理器
pub struct LoginHandler {
    user_repo: Arc<dyn UserRepository>,
    token_service: Arc<dyn TokenService>,
}

impl LoginHandler {
    pub fn new(user_repo: Arc<dyn UserRepository>, token_service: Arc<dyn TokenService>) -> Self {
        Self {
            user_repo,
            token_service,
        }
    }

    /// 处理登录命令
    pub async fn handle(&self, command: LoginCommand) -> Result<LoginResponseDto, LoginError> {
        // 1. 查找用户
        let mut user = self
            .user_repo
            .find_by_username(&command.username)
            .await?
            .ok_or(LoginError::InvalidCredentials)?;

        // 2. 检查账户状态
        if !user.is_active() {
            return Err(LoginError::AccountDisabled);
        }

        // 3. 验证密码
        if !user.check_password(&command.password) {
            return Err(LoginError::InvalidCredentials);
        }

        // 4. 检查 2FA 是否启用
        if user.tfa_enabled() {
            // 生成临时 Token，有效期很短（如 5 分钟）
            // 我们可以在 TokenService 中增加生成临时 token 的方法，或者直接在这里使用现有的
            // 为了区分，我们可以给 TokenClaims 增加 token_type 字段（如果已有）或者在 permissions 中加入特殊标记
            let (temp_token, _, _) = self
                .token_service
                .generate_tokens(
                    user.id().as_uuid().to_string(),
                    command.tenant_id.clone(),
                    vec!["temp_2fa".to_string()],
                    vec![],
                )
                .await
                .map_err(|e| LoginError::TokenError(e.to_string()))?;

            return Ok(LoginResponseDto {
                access_token: None,
                refresh_token: None,
                expires_in: None,
                user: UserDto::from_user(&user),
                requires_2fa: true,
                temp_token: Some(temp_token),
            });
        }

        // 5. 获取用户权限
        let permissions = self.user_repo.get_user_permissions(user.id()).await?;
        let permission_strings: Vec<String> = permissions.iter().map(|p| p.to_string()).collect();
        let role_strings: Vec<String> = user.roles().iter().map(|r| r.to_string()).collect();

        // 6. 生成 Token
        let (access_token, refresh_token, expires_in) = self
            .token_service
            .generate_tokens(
                user.id().as_uuid().to_string(),
                command.tenant_id,
                role_strings.clone(),
                permission_strings,
            )
            .await
            .map_err(|e| LoginError::TokenError(e.to_string()))?;

        // 7. 更新最后登录时间
        user.record_login();
        self.user_repo.save(&mut user).await?;

        // 8. 返回响应
        Ok(LoginResponseDto {
            access_token: Some(access_token),
            refresh_token: Some(refresh_token),
            expires_in: Some(expires_in),
            user: UserDto::from_user(&user),
            requires_2fa: false,
            temp_token: None,
        })
    }
}
