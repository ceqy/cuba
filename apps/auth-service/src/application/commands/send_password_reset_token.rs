//! Send Password Reset Token Handler

use crate::domain::repositories::{UserRepository, VerificationRepository, VerificationTokenData, VerificationTokenType, RepositoryError};
use crate::domain::value_objects::{Email, UserId};
use chrono::{Duration, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct SendPasswordResetTokenCommand {
    pub email: String,
}

pub struct SendPasswordResetTokenHandler {
    user_repo: Arc<dyn UserRepository>,
    verification_repo: Arc<dyn VerificationRepository>,
}

impl SendPasswordResetTokenHandler {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        verification_repo: Arc<dyn VerificationRepository>,
    ) -> Self {
        Self {
            user_repo,
            verification_repo,
        }
    }

    pub async fn handle(&self, command: SendPasswordResetTokenCommand) -> Result<(), crate::domain::errors::DomainError> {
        // 1. 查找用户
        let user = self.user_repo.find_by_email(&command.email).await
            .map_err(|e| crate::domain::errors::DomainError::InfrastructureError(e.to_string()))?
            .ok_or_else(|| crate::domain::errors::DomainError::NotFound("User not found".to_string()))?;

        // 2. 生成随机令牌 (对于密码重置，通常是一个随机且长的字符串)
        let token = Uuid::new_v4().to_string();
        
        // 3. 保存令牌
        let expiration = Utc::now() + Duration::hours(1);
        let token_data = VerificationTokenData {
            id: Uuid::new_v4().to_string(),
            user_id: user.id().clone(),
            token_type: VerificationTokenType::PasswordReset,
            token_hash: token.clone(), // 实际生产环境应存储哈希
            expires_at: expiration,
            created_at: Utc::now(),
            used_at: None,
        };

        self.verification_repo.save(&token_data).await
            .map_err(|e| crate::domain::errors::DomainError::InfrastructureError(e.to_string()))?;

        // 4. 发送邮件 (TODO: 集成邮件服务)
        // 目前仅打印到日志
        println!("PASSWORD RESET TOKEN for {}: {}", command.email, token);

        Ok(())
    }
}
