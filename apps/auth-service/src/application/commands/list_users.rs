//! List Users Handler

use crate::application::dto::UserDto;
use crate::domain::repositories::{UserRepository, RepositoryError};
use crate::domain::value_objects::{UserId, RoleId};
use std::sync::Arc;

pub struct ListUsersCommand {
    pub search: Option<String>,
    pub role_id: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

pub struct ListUsersResponse {
    pub users: Vec<UserDto>,
    pub total: i64,
}

pub struct ListUsersHandler {
    user_repo: Arc<dyn UserRepository>,
}

impl ListUsersHandler {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    pub async fn handle(&self, command: ListUsersCommand) -> Result<ListUsersResponse, RepositoryError> {
        let role_id = if let Some(rid) = command.role_id {
            if rid.is_empty() {
                None
            } else {
                Some(RoleId::parse(&rid).map_err(|_| RepositoryError::DatabaseError("Invalid role_id".to_string()))?)
            }
        } else {
            None
        };

        let users = self.user_repo.find_all(
            command.search.as_deref(),
            role_id.as_ref(),
            command.limit,
            command.offset,
        ).await?;

        let total = self.user_repo.count_all(
            command.search.as_deref(),
            role_id.as_ref(),
        ).await?;

        Ok(ListUsersResponse {
            users: users.into_iter().map(|u| UserDto::from_user(&u)).collect(),
            total,
        })
    }
}
