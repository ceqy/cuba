//! 更新用户资料命令
//!
//! 允许用户更新其显示名称和头像。

use crate::domain::repositories::UserRepository;
use crate::domain::value_objects::UserId;
use crate::domain::DomainError;
use std::sync::Arc;

/// 更新用户资料命令
pub struct UpdateUserProfileCommand {
    pub user_id: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

/// 更新用户资料处理器
pub struct UpdateUserProfileHandler {
    user_repo: Arc<dyn UserRepository>,
}

impl UpdateUserProfileHandler {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    pub async fn handle(&self, command: UpdateUserProfileCommand) -> Result<crate::application::dto::UserDto, DomainError> {
        let user_id = UserId::parse(&command.user_id)
            .map_err(|e| DomainError::InvalidInput(e.to_string()))?;

        let mut user = self.user_repo.find_by_id(&user_id).await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?
            .ok_or_else(|| DomainError::NotFound("用户不存在".to_string()))?;

        // 更新资料
        user.update_profile(command.display_name, command.avatar_url);

        // 保存更新
        self.user_repo.save(&mut user).await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(crate::application::dto::UserDto::from_user(&user))
    }
}
