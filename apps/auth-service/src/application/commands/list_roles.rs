//! 获取角色列表
//!
//! 列出系统中定义的所有角色及其基本信息。

use crate::domain::repositories::RoleRepository;
use crate::application::dto::RoleDto;
use crate::domain::DomainError;
use std::sync::Arc;

/// 获取角色列表处理器
pub struct ListRolesHandler {
    role_repo: Arc<dyn RoleRepository>,
}

pub struct ListRolesCommand {
    pub limit: i64,
    pub offset: i64,
}

pub struct ListRolesResponse {
    pub roles: Vec<RoleDto>,
    pub total: i64,
}

impl ListRolesHandler {
    pub fn new(role_repo: Arc<dyn RoleRepository>) -> Self {
        Self { role_repo }
    }

    pub async fn handle(&self, command: ListRolesCommand) -> Result<ListRolesResponse, DomainError> {
        let roles = self.role_repo.find_all(command.limit, command.offset).await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        
        let total = self.role_repo.count_all().await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        let mut dtos = Vec::new();
        for role in roles {
            dtos.push(RoleDto::from_role(&role));
        }

        Ok(ListRolesResponse { roles: dtos, total })
    }
}
