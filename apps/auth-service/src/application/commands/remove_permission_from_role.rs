//! 移除角色权限命令
//!
//! 从指定角色中移除一个现有的权限绑定。

use crate::domain::repositories::RoleRepository;
use crate::domain::value_objects::{RoleId, PermissionId};
use crate::domain::DomainError;
use std::sync::Arc;

/// 移除角色权限命令
pub struct RemovePermissionFromRoleCommand {
    pub role_id: String,
    pub permission_id: String,
}

/// 移除角色权限处理器
pub struct RemovePermissionFromRoleHandler {
    role_repo: Arc<dyn RoleRepository>,
}

impl RemovePermissionFromRoleHandler {
    pub fn new(role_repo: Arc<dyn RoleRepository>) -> Self {
        Self { role_repo }
    }

    pub async fn handle(&self, command: RemovePermissionFromRoleCommand) -> Result<(), DomainError> {
        let role_id = RoleId::parse(&command.role_id)
            .map_err(|e| DomainError::InvalidInput(e.to_string()))?;
        let permission_id = PermissionId::parse(&command.permission_id)
            .map_err(|e| DomainError::InvalidInput(e.to_string()))?;

        // 查找角色
        let mut role = self.role_repo.find_by_id(&role_id).await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?
            .ok_or_else(|| DomainError::NotFound("角色不存在".to_string()))?;

        // 注意：Role 聚合根目前是存储 Vec<Permission> 而不是 Vec<PermissionId>
        // 我需要先根据 PermissionId 找到 Permission 值对象，然后从聚合根中移除
        let permission_to_remove = self.role_repo.find_permission_by_id(&permission_id).await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?
            .ok_or_else(|| DomainError::NotFound("权限不存在".to_string()))?;

        // 从角色聚合根中移除权限
        role.remove_permission(&permission_to_remove)?;

        // 保存角色状态
        self.role_repo.save(&mut role).await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }
}
