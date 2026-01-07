//! Social Auth Service (Infrastructure)
//!
//! 处理第三方身份提供商的交互。

use async_trait::async_trait;
use crate::domain::DomainError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SocialUserProfile {
    pub id: String,
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
}

#[async_trait]
pub trait SocialAuthProvider: Send + Sync {
    async fn verify_code(&self, code: &str, redirect_uri: &str) -> Result<SocialUserProfile, DomainError>;
}

/// Google Auth Provider
pub struct GoogleAuthProvider {
    client_id: String,
    client_secret: String,
    http_client: reqwest::Client,
}

impl GoogleAuthProvider {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            http_client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl SocialAuthProvider for GoogleAuthProvider {
    async fn verify_code(&self, code: &str, redirect_uri: &str) -> Result<SocialUserProfile, DomainError> {
        // TODO: Implement actual code exchange
        // For now, we mock it to avoid needing real credentials in development
        
        if code == "mock_code" {
             Ok(SocialUserProfile {
                id: "123456789".to_string(),
                email: "mock_user@gmail.com".to_string(),
                name: "Mock User".to_string(),
                picture: Some("https://example.com/avatar.jpg".to_string()),
            })
        } else {
             Err(DomainError::AuthenticationFailed("Invalid social auth code".to_string()))
        }
    }
}

pub struct SocialAuthService {
    google_provider: GoogleAuthProvider,
}

impl SocialAuthService {
    pub fn new(google_client_id: String, google_client_secret: String) -> Self {
        Self {
            google_provider: GoogleAuthProvider::new(google_client_id, google_client_secret),
        }
    }

    pub fn get_provider(&self, provider_name: &str) -> Result<&dyn SocialAuthProvider, DomainError> {
        match provider_name {
            "google" => Ok(&self.google_provider),
            _ => Err(DomainError::InvalidInput(format!("Unsupported provider: {}", provider_name))),
        }
    }
}
