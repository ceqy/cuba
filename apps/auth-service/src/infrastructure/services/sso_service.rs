use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use crate::domain::errors::DomainError;

#[derive(Debug, Clone)]
pub struct SSOUserProfile {
    pub email: String,
    pub display_name: Option<String>,
    pub provider_id: String, // User ID in the IdP
}

#[async_trait]
pub trait SSOProvider: Send + Sync {
    async fn verify_assertion(&self, assertion: &str) -> Result<SSOUserProfile, DomainError>;
}

pub struct MockSSOProvider {
    name: String,
}

impl MockSSOProvider {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }
}

#[async_trait]
impl SSOProvider for MockSSOProvider {
    async fn verify_assertion(&self, assertion: &str) -> Result<SSOUserProfile, DomainError> {
        // Mock verification: assume assertion is just the email for testing purposes
        // In reality, this would verify SAML signature or OIDC ID Token
        if assertion.contains("@") {
            Ok(SSOUserProfile {
                email: assertion.to_string(),
                display_name: Some(format!("User via {}", self.name)),
                provider_id: format!("mock-id-{}", uuid::Uuid::new_v4()),
            })
        } else {
            Err(DomainError::InvalidInput("Invalid assertion (mock)".to_string()))
        }
    }
}

pub struct SSOService {
    providers: HashMap<String, Arc<dyn SSOProvider>>,
}

impl SSOService {
    pub fn new() -> Self {
        let mut providers: HashMap<String, Arc<dyn SSOProvider>> = HashMap::new();
        // Register default mock providers
        providers.insert("saml".to_string(), Arc::new(MockSSOProvider::new("SAML Mock")));
        providers.insert("oidc".to_string(), Arc::new(MockSSOProvider::new("OIDC Mock")));
        providers.insert("okta".to_string(), Arc::new(MockSSOProvider::new("Okta")));
        
        Self { providers }
    }

    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn SSOProvider>> {
        self.providers.get(name).cloned()
    }
}
