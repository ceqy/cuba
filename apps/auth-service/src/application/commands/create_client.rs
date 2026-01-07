//! Create Client Handler

use crate::domain::repositories::{ClientRepository, ClientData, RepositoryError};
use std::sync::Arc;

pub struct CreateClientCommand {
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub grant_types: Vec<String>,
    pub scopes: Vec<String>,
}

pub struct CreateClientResponse {
    pub client: ClientData,
    pub client_secret_plain: String,
}

pub struct CreateClientHandler {
    client_repo: Arc<dyn ClientRepository>,
}

impl CreateClientHandler {
    pub fn new(client_repo: Arc<dyn ClientRepository>) -> Self {
        Self { client_repo }
    }

    pub async fn handle(&self, command: CreateClientCommand) -> Result<CreateClientResponse, RepositoryError> {
        let client_id = uuid::Uuid::new_v4().to_string();
        let client_secret_plain = uuid::Uuid::new_v4().to_string().replace("-", ""); 
        
        // In a real application, hash the secret using Argon2 or similar.
        // For this demo, we store it plain or simple hash (TODO: Use generic hasher)
        let client_secret_hash = client_secret_plain.clone(); 

        let client = ClientData {
            client_id,
            client_secret: client_secret_hash,
            name: command.name,
            redirect_uris: command.redirect_uris,
            grant_types: command.grant_types,
            scopes: command.scopes,
            created_at: chrono::Utc::now(),
        };

        self.client_repo.save(&client).await?;

        Ok(CreateClientResponse {
            client,
            client_secret_plain,
        })
    }
}
