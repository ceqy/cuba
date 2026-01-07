//! Create Role Command

use crate::application::dto::RoleDto;
use crate::domain::aggregates::Role;
use crate::domain::repositories::{RoleRepository, RepositoryError};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug)]
pub struct CreateRoleCommand {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Error, Debug)]
pub enum CreateRoleError {
    #[error("Role already exists: {0}")]
    AlreadyExists(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Domain error: {0}")]
    DomainError(String),
}

pub struct CreateRoleHandler {
    role_repo: Arc<dyn RoleRepository>,
}

impl CreateRoleHandler {
    pub fn new(role_repo: Arc<dyn RoleRepository>) -> Self {
        Self { role_repo }
    }

    pub async fn handle(&self, command: CreateRoleCommand) -> Result<RoleDto, CreateRoleError> {
        // Check if role exists
        if let Some(_) = self.role_repo.find_by_name(&command.name).await
            .map_err(|e| CreateRoleError::RepositoryError(e.to_string()))? 
        {
            return Err(CreateRoleError::AlreadyExists(command.name));
        }

        // Create domain object
        let mut role = Role::create(command.name, command.description)
            .map_err(|e| CreateRoleError::DomainError(e.to_string()))?;

        // Save to repository
        self.role_repo.save(&mut role).await
            .map_err(|e| CreateRoleError::RepositoryError(e.to_string()))?;

        Ok(RoleDto::from_role(&role))
    }
}

