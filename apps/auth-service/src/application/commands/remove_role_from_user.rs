//! 移除用户角色命令
//!
//! 从用户账户中撤销一个已分配的角色。

use crate::domain::repositories::{UserRepository, RoleRepository};
use crate::domain::value_objects::{UserId, RoleId};
use crate::domain::DomainError;
use std::sync::Arc;

/// 移除用户角色命令
pub struct RemoveRoleFromUserCommand {
    pub user_id: String,
    pub role_id: String,
}

/// 移除用户角色处理器
pub struct RemoveRoleFromUserHandler {
    user_repo: Arc<dyn UserRepository>,
    role_repo: Arc<dyn RoleRepository>,
}

impl RemoveRoleFromUserHandler {
    pub fn new(user_repo: Arc<dyn UserRepository>, role_repo: Arc<dyn RoleRepository>) -> Self {
        Self { user_repo, role_repo }
    }

    pub async fn handle(&self, command: RemoveRoleFromUserCommand) -> Result<(), DomainError> {
        let user_id = UserId::parse(&command.user_id)
            .map_err(|e| DomainError::InvalidInput(e.to_string()))?;
        let role_id = RoleId::parse(&command.role_id)
            .map_err(|e| DomainError::InvalidInput(e.to_string()))?;

        // 检查用户是否存在
        let mut user = self.user_repo.find_by_id(&user_id).await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?
            .ok_or_else(|| DomainError::NotFound("用户不存在".to_string()))?;

        // 检查角色是否存在
        let _role = self.role_repo.find_by_id(&role_id).await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?
            .ok_or_else(|| DomainError::NotFound("角色不存在".to_string()))?;

        // 从用户聚合根中移除角色
        user.remove_role(&role_id)?;

        // 保存用户状态
        self.user_repo.save(&mut user).await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }
}
