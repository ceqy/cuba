//! Revoke Session Handler

use crate::domain::repositories::{SessionRepository, RepositoryError};
use crate::domain::value_objects::UserId;
use std::sync::Arc;

pub struct RevokeSessionCommand {
    pub user_id: String,
    pub session_id: String, // "all" or specific ID
}

pub struct RevokeSessionHandler {
    session_repo: Arc<dyn SessionRepository>,
}

impl RevokeSessionHandler {
    pub fn new(session_repo: Arc<dyn SessionRepository>) -> Self {
        Self { session_repo }
    }

    pub async fn handle(&self, command: RevokeSessionCommand) -> Result<(), RepositoryError> {
        if command.session_id == "all" {
            let user_id = UserId::parse(&command.user_id)
                .map_err(|_| RepositoryError::DatabaseError("Invalid user_id".to_string()))?;
            self.session_repo.delete_all_for_user(&user_id).await?;
        } else {
             if let Some(session) = self.session_repo.find_by_id(&command.session_id).await? {
                 if session.user_id.to_string() == command.user_id {
                     self.session_repo.delete(&command.session_id).await?;
                 } else {
                     return Err(RepositoryError::NotFound); // Access denied treated as not found
                 }
             } else {
                 return Err(RepositoryError::NotFound);
             }
        }
        
        Ok(())
    }
}
