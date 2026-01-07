//! Register Command
//!
//! 处理用户注册用例。

use crate::application::dto::UserDto;
use crate::domain::aggregates::User;
use crate::domain::repositories::{RepositoryError, UserRepository};
use std::sync::Arc;
use thiserror::Error;

/// 注册命令
#[derive(Debug, Clone)]
pub struct RegisterCommand {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// 注册错误
#[derive(Error, Debug)]
pub enum RegisterError {
    #[error("Username already exists")]
    UsernameExists,

    #[error("Email already registered")]
    EmailExists,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Repository error: {0}")]
    RepositoryError(#[from] RepositoryError),

    #[error("Domain error: {0}")]
    DomainError(String),
}

/// 注册命令处理器
pub struct RegisterHandler {
    user_repo: Arc<dyn UserRepository>,
}

impl RegisterHandler {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    /// 处理注册命令
    pub async fn handle(&self, command: RegisterCommand) -> Result<UserDto, RegisterError> {
        // 1. 检查用户名是否已存在
        if self.user_repo.username_exists(&command.username).await? {
            return Err(RegisterError::UsernameExists);
        }

        // 2. 检查邮箱是否已注册
        if self.user_repo.email_exists(&command.email).await? {
            return Err(RegisterError::EmailExists);
        }

        // 3. 创建用户聚合根
        let mut user = User::register(command.username, command.email, &command.password)
            .map_err(|e| RegisterError::DomainError(e.to_string()))?;

        // 4. 持久化用户
        self.user_repo.save(&mut user).await?;

        // 5. 返回 DTO
        Ok(UserDto::from_user(&user))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::aggregates::User;
    use crate::domain::value_objects::{Permission, UserId, RoleId}; // Added RoleId
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mock UserRepository for testing
    struct MockUserRepository {
        users: Mutex<HashMap<String, User>>,
    }

    impl MockUserRepository {
        fn new() -> Self {
            Self {
                users: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn save(&self, user: &mut User) -> Result<(), RepositoryError> {
            let mut users = self.users.lock().unwrap();
            users.insert(user.username().to_string(), user.clone());
            Ok(())
        }

        async fn find_by_id(&self, _id: &UserId) -> Result<Option<User>, RepositoryError> {
            Ok(None)
        }

        async fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError> {
            let users = self.users.lock().unwrap();
            Ok(users.get(username).cloned())
        }

        async fn find_by_email(&self, _email: &str) -> Result<Option<User>, RepositoryError> {
            Ok(None)
        }

        async fn username_exists(&self, username: &str) -> Result<bool, RepositoryError> {
            let users = self.users.lock().unwrap();
            Ok(users.contains_key(username))
        }

        async fn email_exists(&self, _email: &str) -> Result<bool, RepositoryError> {
            Ok(false)
        }

        async fn get_user_permissions(
            &self,
            _id: &UserId,
        ) -> Result<Vec<Permission>, RepositoryError> {
            Ok(vec![])
        }

        async fn delete(&self, _id: &UserId) -> Result<(), RepositoryError> {
            Ok(())
        }

        async fn find_all(
            &self,
            _search: Option<&str>,
            _role_id: Option<&RoleId>,
            _limit: i64,
            _offset: i64,
        ) -> Result<Vec<User>, RepositoryError> {
            Ok(vec![])
        }

        async fn count_all(
            &self,
            _search: Option<&str>,
            _role_id: Option<&RoleId>,
        ) -> Result<i64, RepositoryError> {
            Ok(0)
        }
    }

    #[tokio::test]
    async fn test_register_success() {
        let repo = Arc::new(MockUserRepository::new());
        let handler = RegisterHandler::new(repo);

        let command = RegisterCommand {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "Password123".to_string(),
        };

        let result = handler.handle(command).await;
        assert!(result.is_ok());
        let user_dto = result.unwrap();
        assert_eq!(user_dto.username, "testuser");
    }

    #[tokio::test]
    async fn test_register_duplicate_username() {
        let repo = Arc::new(MockUserRepository::new());
        let handler = RegisterHandler::new(repo.clone());

        // First registration
        let command1 = RegisterCommand {
            username: "testuser".to_string(),
            email: "test1@example.com".to_string(),
            password: "Password123".to_string(),
        };
        handler.handle(command1).await.unwrap();

        // Duplicate username
        let command2 = RegisterCommand {
            username: "testuser".to_string(),
            email: "test2@example.com".to_string(),
            password: "Password123".to_string(),
        };
        let result = handler.handle(command2).await;
        assert!(matches!(result, Err(RegisterError::UsernameExists)));
    }
}
