//! List User Sessions Handler

use crate::domain::repositories::{SessionData, SessionRepository, RepositoryError};
use crate::domain::value_objects::UserId;
use std::sync::Arc;

pub struct ListUserSessionsCommand {
    pub user_id: String,
}

pub struct ListUserSessionsResponse {
    pub sessions: Vec<SessionData>,
}

pub struct ListUserSessionsHandler {
    session_repo: Arc<dyn SessionRepository>,
}

impl ListUserSessionsHandler {
    pub fn new(session_repo: Arc<dyn SessionRepository>) -> Self {
        Self { session_repo }
    }

    pub async fn handle(&self, command: ListUserSessionsCommand) -> Result<ListUserSessionsResponse, RepositoryError> {
        let user_id = UserId::parse(&command.user_id)
            .map_err(|_| RepositoryError::DatabaseError("Invalid user_id".to_string()))?;
        
        let sessions = self.session_repo.find_by_user(&user_id).await?;
        
        Ok(ListUserSessionsResponse { sessions })
    }
}
