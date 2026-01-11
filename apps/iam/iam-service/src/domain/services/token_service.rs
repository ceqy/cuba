use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (User ID)
    pub exp: usize,  // Expiration time
    pub iat: usize,  // Issued at
    pub tenant_id: Option<String>,
}

#[derive(Debug)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

#[async_trait]
pub trait TokenService: Send + Sync {
    fn generate_tokens(&self, user_id: &str, tenant_id: Option<String>) -> Result<TokenPair>;
    fn validate_token(&self, token: &str) -> Result<Claims>;
}
