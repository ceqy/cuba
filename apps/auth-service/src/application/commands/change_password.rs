//! 修改密码命令
//!
//! 允许用户在验证旧密码后更新为新密码。

use crate::domain::repositories::UserRepository;
use crate::domain::value_objects::UserId;
use crate::domain::aggregates::User;
use crate::domain::DomainError;
use bcrypt::{hash, verify, DEFAULT_COST};
use std::sync::Arc;

/// 修改密码命令
pub struct ChangePasswordCommand {
    pub user_id: String,
    pub current_password: String,
    pub new_password: String,
}

/// 修改密码处理器
pub struct ChangePasswordHandler {
    user_repo: Arc<dyn UserRepository>,
}

impl ChangePasswordHandler {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    pub async fn handle(&self, command: ChangePasswordCommand) -> Result<(), DomainError> {
        let user_id = UserId::parse(&command.user_id)
            .map_err(|e| DomainError::InvalidInput(e.to_string()))?;

        let mut user = self.user_repo.find_by_id(&user_id).await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?
            .ok_or_else(|| DomainError::NotFound("用户不存在".to_string()))?;

        // 验证当前密码
        if !verify(&command.current_password, user.password_hash()).map_err(|e| DomainError::InternalError(e.to_string()))? {
            return Err(DomainError::AuthenticationFailed("当前密码错误".to_string()));
        }

        // 验证新密码强度 (简单示例)
        if command.new_password.len() < 8 {
            return Err(DomainError::InvalidInput("新密码长度必须至少为 8 位".to_string()));
        }

        // 更新用户聚合根
        user.update_password(&command.new_password)?;

        // 保存更新
        self.user_repo.save(&mut user).await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }
}
