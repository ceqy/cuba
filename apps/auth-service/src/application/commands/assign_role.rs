//! Assign Role to User Command

use crate::domain::repositories::{UserRepository, RoleRepository, RepositoryError};
use crate::domain::value_objects::{UserId, RoleId};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug)]
pub struct AssignRoleCommand {
    pub user_id: String,
    pub role_id: String,
}

#[derive(Error, Debug)]
pub enum AssignRoleError {
    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Role not found: {0}")]
    RoleNotFound(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Domain error: {0}")]
    DomainError(String),

    #[error("Invalid ID format: {0}")]
    InvalidId(String),
}

pub struct AssignRoleHandler {
    user_repo: Arc<dyn UserRepository>,
    role_repo: Arc<dyn RoleRepository>,
}

impl AssignRoleHandler {
    pub fn new(user_repo: Arc<dyn UserRepository>, role_repo: Arc<dyn RoleRepository>) -> Self {
        Self { user_repo, role_repo }
    }

    pub async fn handle(&self, command: AssignRoleCommand) -> Result<(), AssignRoleError> {
        let user_id = UserId::from(
            uuid::Uuid::parse_str(&command.user_id)
                .map_err(|_| AssignRoleError::InvalidId(command.user_id.clone()))?
        );

        let role_id = RoleId::from(
            uuid::Uuid::parse_str(&command.role_id)
                .map_err(|_| AssignRoleError::InvalidId(command.role_id.clone()))?
        );

        // 1. Verify role exists
        let _role = self.role_repo.find_by_id(&role_id).await
            .map_err(|e| AssignRoleError::RepositoryError(e.to_string()))?
            .ok_or_else(|| AssignRoleError::RoleNotFound(command.role_id))?;

        // 2. Load user
        let mut user = self.user_repo.find_by_id(&user_id).await
            .map_err(|e| AssignRoleError::RepositoryError(e.to_string()))?
            .ok_or_else(|| AssignRoleError::UserNotFound(command.user_id))?;

        // 3. Assign role
        user.assign_role(role_id)
            .map_err(|e| AssignRoleError::DomainError(e.to_string()))?;

        // 4. Save user
        self.user_repo.save(&mut user).await
            .map_err(|e| AssignRoleError::RepositoryError(e.to_string()))?;

        Ok(())
    }
}
