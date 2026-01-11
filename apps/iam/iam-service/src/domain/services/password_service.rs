use async_trait::async_trait;
use anyhow::Result;

#[async_trait]
pub trait PasswordService: Send + Sync {
    async fn hash(&self, plain_password: &str) -> Result<String>;
    async fn verify(&self, plain_password: &str, hashed_password: &str) -> Result<bool>;
}
