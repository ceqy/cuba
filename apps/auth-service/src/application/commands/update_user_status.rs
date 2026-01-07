//! 更新用户状态命令
//!
//! 允许管理员启用或禁用用户账户。

use crate::domain::repositories::UserRepository;
use crate::domain::value_objects::UserId;
use crate::domain::DomainError;
use std::sync::Arc;

/// 更新用户状态命令
pub struct UpdateUserStatusCommand {
    pub user_id: String,
    pub is_active: bool,
}

/// 更新用户状态处理器
pub struct UpdateUserStatusHandler {
    user_repo: Arc<dyn UserRepository>,
}

impl UpdateUserStatusHandler {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    pub async fn handle(&self, command: UpdateUserStatusCommand) -> Result<(), DomainError> {
        let user_id = UserId::parse(&command.user_id)
            .map_err(|e| DomainError::InvalidInput(e.to_string()))?;

        let mut user = self.user_repo.find_by_id(&user_id).await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?
            .ok_or_else(|| DomainError::NotFound("用户不存在".to_string()))?;

        if command.is_active {
            user.activate();
        } else {
            user.deactivate();
        }

        // 保存更新
        self.user_repo.save(&mut user).await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }
}
