//! Verify Email Handler

use crate::domain::repositories::{UserRepository, VerificationRepository, VerificationTokenType, RepositoryError};
use crate::domain::errors::DomainError;
use std::sync::Arc;
use chrono::Utc;

pub struct VerifyEmailCommand {
    pub token: String,
}

pub struct VerifyEmailHandler {
    user_repo: Arc<dyn UserRepository>,
    verification_repo: Arc<dyn VerificationRepository>,
}

impl VerifyEmailHandler {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        verification_repo: Arc<dyn VerificationRepository>,
    ) -> Self {
        Self {
            user_repo,
            verification_repo,
        }
    }

    pub async fn handle(&self, command: VerifyEmailCommand) -> Result<(), DomainError> {
        // 1. 验证令牌
        let token_data = self.verification_repo.find_by_hash(&command.token, VerificationTokenType::EmailVerification).await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?
            .ok_or_else(|| DomainError::InvalidInput("Invalid or expired token".to_string()))?;

        if token_data.used_at.is_some() || token_data.expires_at < Utc::now() {
            return Err(DomainError::InvalidInput("Invalid or expired token".to_string()));
        }

        // 2. 查找用户
        let mut user = self.user_repo.find_by_id(&token_data.user_id).await
            .map_err(|e| DomainError::InvalidInput(e.to_string()))?
            .ok_or(DomainError::UserNotFound)?;

        // 3. 验证邮箱
        user.verify_email();

        // 4. 保存用户
        self.user_repo.save(&mut user).await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        // 5. 标记令牌为已使用
        self.verification_repo.mark_as_used(&command.token).await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }
}
