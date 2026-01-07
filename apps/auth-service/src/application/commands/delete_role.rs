//! 删除角色命令
//!
//! 从系统中完全移除一个角色并撤销所有关联。

use crate::domain::repositories::RoleRepository;
use crate::domain::value_objects::RoleId;
use crate::domain::DomainError;
use std::sync::Arc;

/// 删除角色命令
pub struct DeleteRoleCommand {
    pub role_id: String,
}

/// 删除角色处理器
pub struct DeleteRoleHandler {
    role_repo: Arc<dyn RoleRepository>,
}

impl DeleteRoleHandler {
    pub fn new(role_repo: Arc<dyn RoleRepository>) -> Self {
        Self { role_repo }
    }

    pub async fn handle(&self, command: DeleteRoleCommand) -> Result<(), DomainError> {
        let role_id = RoleId::parse(&command.role_id)
            .map_err(|e| DomainError::InvalidInput(e.to_string()))?;

        // 检查角色是否存在
        let role = self.role_repo.find_by_id(&role_id).await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?
            .ok_or_else(|| DomainError::NotFound("角色不存在".to_string()))?;

        // TODO: 检查是否有用户正在使用该角色，如果需要限制删除

        self.role_repo.delete(&role.id()).await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }
}
