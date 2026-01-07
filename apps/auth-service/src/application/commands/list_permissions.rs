//! 获取权限列表
//!
//! 列出系统中定义的所有权限及其资源和操作。

use crate::domain::repositories::RoleRepository;
use crate::application::dto::PermissionDto;
use crate::domain::DomainError;
use std::sync::Arc;

/// 获取权限列表处理器
pub struct ListPermissionsHandler {
    role_repo: Arc<dyn RoleRepository>,
}

pub struct ListPermissionsCommand {
    pub limit: i64,
    pub offset: i64,
}

pub struct ListPermissionsResponse {
    pub permissions: Vec<PermissionDto>,
    pub total: i64,
}

impl ListPermissionsHandler {
    pub fn new(role_repo: Arc<dyn RoleRepository>) -> Self {
        Self { role_repo }
    }

    pub async fn handle(&self, command: ListPermissionsCommand) -> Result<ListPermissionsResponse, DomainError> {
        let permissions = self.role_repo.find_all_permissions(command.limit, command.offset).await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        
        let total = self.role_repo.count_all_permissions().await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        let dtos = permissions
            .into_iter()
            .map(|(id, p)| PermissionDto {
                permission_id: id.to_string(),
                resource: p.resource().to_string(),
                action: p.action().to_string(),
                description: None, 
            })
            .collect();
            
        Ok(ListPermissionsResponse { permissions: dtos, total })
    }
}
