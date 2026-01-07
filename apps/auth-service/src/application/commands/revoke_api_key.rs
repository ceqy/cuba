//! Revoke API Key Handler

use crate::domain::repositories::{ApiKeyRepository, RepositoryError};
use std::sync::Arc;

pub struct RevokeAPIKeyCommand {
    pub key_id: String,
}

pub struct RevokeAPIKeyHandler {
    api_key_repo: Arc<dyn ApiKeyRepository>,
}

impl RevokeAPIKeyHandler {
    pub fn new(api_key_repo: Arc<dyn ApiKeyRepository>) -> Self {
        Self { api_key_repo }
    }

    pub async fn handle(&self, command: RevokeAPIKeyCommand) -> Result<(), RepositoryError> {
        self.api_key_repo.revoke(&command.key_id).await
    }
}
