use async_trait::async_trait;
use crate::domain::entities::{OAuthClient, AuthorizationCode, RefreshToken};

#[async_trait]
pub trait ClientRepository: Send + Sync {
    async fn find_by_id(&self, client_id: &str) -> Result<Option<OAuthClient>, anyhow::Error>;
    async fn save(&self, client: &OAuthClient) -> Result<(), anyhow::Error>;
    async fn delete(&self, client_id: &str) -> Result<(), anyhow::Error>;
    async fn list_all(&self) -> Result<Vec<OAuthClient>, anyhow::Error>;
}

#[async_trait]
pub trait AuthCodeRepository: Send + Sync {
    async fn find_by_code(&self, code: &str) -> Result<Option<AuthorizationCode>, anyhow::Error>;
    async fn save(&self, code: &AuthorizationCode) -> Result<(), anyhow::Error>;
    async fn delete(&self, code: &str) -> Result<(), anyhow::Error>;
}

#[async_trait]
pub trait RefreshTokenRepository: Send + Sync {
    async fn find_by_token(&self, token: &str) -> Result<Option<RefreshToken>, anyhow::Error>;
    async fn save(&self, token: &RefreshToken) -> Result<(), anyhow::Error>;
    async fn delete(&self, token: &str) -> Result<(), anyhow::Error>;
}
