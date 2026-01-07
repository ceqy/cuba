//! Add Permission to Role Command

use crate::domain::repositories::{RoleRepository, RepositoryError};
use crate::domain::value_objects::{Permission, RoleId, PermissionId};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug)]
pub struct AddPermissionCommand {
    pub role_id: String,
    pub permission_id: String,
}

#[derive(Error, Debug)]
pub enum AddPermissionError {
    #[error("Role not found: {0}")]
    RoleNotFound(String),

    #[error("Permission not found: {0}")]
    PermissionNotFound(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Domain error: {0}")]
    DomainError(String),

    #[error("Invalid ID format: {0}")]
    InvalidId(String),
}

pub struct AddPermissionHandler {
    role_repo: Arc<dyn RoleRepository>,
}

impl AddPermissionHandler {
    pub fn new(role_repo: Arc<dyn RoleRepository>) -> Self {
        Self { role_repo }
    }

    pub async fn handle(&self, command: AddPermissionCommand) -> Result<(), AddPermissionError> {
        let role_id = RoleId::from(
            uuid::Uuid::parse_str(&command.role_id)
                .map_err(|_| AddPermissionError::InvalidId(command.role_id.clone()))?
        );

        let permission_id = PermissionId::from(
            uuid::Uuid::parse_str(&command.permission_id)
                .map_err(|_| AddPermissionError::InvalidId(command.permission_id.clone()))?
        );

        // 1. Load role
        let mut role = self.role_repo.find_by_id(&role_id).await
            .map_err(|e| AddPermissionError::RepositoryError(e.to_string()))?
            .ok_or_else(|| AddPermissionError::RoleNotFound(command.role_id))?;

        // 2. Load permission (We need a way to find permission by ID)
        // For now, let's assume the repository can get us the permission resource/action by ID
        // or we simply lookup it up.
        let permission = self.role_repo.find_permission_by_id(&permission_id).await
            .map_err(|e| AddPermissionError::RepositoryError(e.to_string()))?
            .ok_or_else(|| AddPermissionError::PermissionNotFound(command.permission_id))?;

        // 3. Add permission
        role.add_permission(permission)
            .map_err(|e| AddPermissionError::DomainError(e.to_string()))?;

        // 4. Save role
        self.role_repo.save(&mut role).await
            .map_err(|e| AddPermissionError::RepositoryError(e.to_string()))?;

        Ok(())
    }
}
