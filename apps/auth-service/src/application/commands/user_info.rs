//! OIDC UserInfo Endpoint Handler
//!
//! 处理 OpenID Connect UserInfo 请求。
//! 根据 Access Token 获取当前用户的详细信息。

use crate::application::dto::UserDto;
use crate::domain::repositories::UserRepository;
use crate::domain::services::{TokenService, TokenError};
use crate::domain::value_objects::UserId;
use crate::domain::DomainError;
use std::sync::Arc;

/// UserInfo 查询请求
/// 
/// 虽然是查询，但在本项目中统一放在 commands 目录下
pub struct UserInfoCommand {
    pub access_token: String,
}

/// UserInfo 处理器
pub struct UserInfoHandler {
    user_repo: Arc<dyn UserRepository>,
    token_service: Arc<dyn TokenService>,
}

impl UserInfoHandler {
    pub fn new(user_repo: Arc<dyn UserRepository>, token_service: Arc<dyn TokenService>) -> Self {
        Self { user_repo, token_service }
    }

    pub async fn handle(&self, command: UserInfoCommand) -> Result<UserDto, DomainError> {
        // 1. 验证 Token
        let claims = self.token_service.validate_token(&command.access_token)
            .map_err(|e| match e {
                TokenError::Expired => DomainError::TokenExpired,
                TokenError::Revoked => DomainError::TokenRevoked,
                _ => DomainError::TokenInvalid,
            })?;

        // 2. 解析 User ID
        let user_id = UserId::parse(&claims.sub)
            .map_err(|e| DomainError::InvalidInput(e.to_string()))?;

        // 3. 获取用户信息
        let user = self.user_repo.find_by_id(&user_id).await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?
            .ok_or_else(|| DomainError::NotFound("用户不存在".to_string()))?;

        // 4. 转换为 DTO
        Ok(UserDto::from_user(&user))
    }
}
