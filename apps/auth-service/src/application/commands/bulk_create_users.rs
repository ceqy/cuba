//! Bulk Create Users Handler

use crate::application::dto::UserDto;
use crate::application::commands::register::{RegisterCommand, RegisterHandler};
use crate::domain::repositories::UserRepository;
use std::sync::Arc;
use thiserror::Error;

pub struct BulkCreateUsersCommand {
    pub users: Vec<RegisterCommand>,
}

pub struct BulkCreateUsersResponse {
    pub created_users: Vec<UserDto>,
    pub errors: Vec<String>,
}

pub struct BulkCreateUsersHandler {
    register_handler: RegisterHandler,
}

impl BulkCreateUsersHandler {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self {
            register_handler: RegisterHandler::new(user_repo),
        }
    }

    pub async fn handle(&self, command: BulkCreateUsersCommand) -> BulkCreateUsersResponse {
        let mut created_users = Vec::new();
        let mut errors = Vec::new();

        for user_cmd in command.users {
            match self.register_handler.handle(user_cmd).await {
                Ok(dto) => created_users.push(dto),
                Err(e) => errors.push(e.to_string()),
            }
        }

        BulkCreateUsersResponse {
            created_users,
            errors,
        }
    }
}
