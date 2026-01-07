//! List API Keys Handler

use crate::domain::repositories::{ApiKeyRepository, ApiKeyData, RepositoryError};
use crate::domain::value_objects::UserId;
use std::sync::Arc;

pub struct ListAPIKeysCommand {
    pub user_id: String,
    pub limit: i64,
    pub offset: i64,
}

pub struct ListAPIKeysResponse {
    pub keys: Vec<ApiKeyData>,
    pub total: i64,
}

pub struct ListAPIKeysHandler {
    api_key_repo: Arc<dyn ApiKeyRepository>,
}

impl ListAPIKeysHandler {
    pub fn new(api_key_repo: Arc<dyn ApiKeyRepository>) -> Self {
        Self { api_key_repo }
    }

    pub async fn handle(&self, command: ListAPIKeysCommand) -> Result<ListAPIKeysResponse, RepositoryError> {
        let user_id = UserId::parse(&command.user_id)
            .map_err(|_| RepositoryError::DatabaseError("Invalid user_id".to_string()))?;
        let keys = self.api_key_repo.find_by_user(&user_id, command.limit, command.offset).await?;
        let total = self.api_key_repo.count_by_user(&user_id).await?;
        Ok(ListAPIKeysResponse { keys, total })
    }
}
