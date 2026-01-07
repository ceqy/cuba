//! List Clients Handler

use crate::domain::repositories::{ClientRepository, ClientData, RepositoryError};
use std::sync::Arc;

pub struct ListClientsCommand {
    pub limit: i64,
    pub offset: i64,
}

pub struct ListClientsResponse {
    pub clients: Vec<ClientData>,
    pub total: i64,
}

pub struct ListClientsHandler {
    client_repo: Arc<dyn ClientRepository>,
}

impl ListClientsHandler {
    pub fn new(client_repo: Arc<dyn ClientRepository>) -> Self {
        Self { client_repo }
    }

    pub async fn handle(&self, command: ListClientsCommand) -> Result<ListClientsResponse, RepositoryError> {
        let clients = self.client_repo.find_all(command.limit, command.offset).await?;
        let total = self.client_repo.count_all().await?;
        Ok(ListClientsResponse { clients, total })
    }
}
