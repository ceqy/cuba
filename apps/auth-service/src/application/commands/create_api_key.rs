//! Create API Key Handler

use crate::domain::repositories::{ApiKeyRepository, ApiKeyData, RepositoryError};
use crate::domain::value_objects::UserId;
use chrono::{Duration, Utc};
use std::sync::Arc;
use uuid::Uuid;
use ring::rand::{SecureRandom, SystemRandom};
use base64::{engine::general_purpose, Engine as _};

pub struct CreateAPIKeyCommand {
    pub user_id: String,
    pub tenant_id: String,
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<chrono::DateTime<Utc>>,
}

pub struct CreateAPIKeyResponse {
    pub key_id: String,
    pub api_key: String, // Plaintext, only once
    pub prefix: String,
}

pub struct CreateAPIKeyHandler {
    api_key_repo: Arc<dyn ApiKeyRepository>,
}

impl CreateAPIKeyHandler {
    pub fn new(api_key_repo: Arc<dyn ApiKeyRepository>) -> Self {
        Self { api_key_repo }
    }

    pub async fn handle(&self, command: CreateAPIKeyCommand) -> Result<CreateAPIKeyResponse, RepositoryError> {
        // 1. 生成随机 API Key
        let mut key_bytes = [0u8; 32];
        let rng = SystemRandom::new();
        rng.fill(&mut key_bytes).map_err(|_| RepositoryError::DatabaseError("RNG failure".to_string()))?;
        
        let raw_key = general_purpose::URL_SAFE_NO_PAD.encode(key_bytes);
        
        // 2. 生成前缀和哈希
        let prefix = format!("ak_{}", &raw_key[0..8]);
        
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(raw_key.as_bytes());
        let key_hash = format!("{:x}", hasher.finalize());

        // 3. 保存到数据库
        let key_id = Uuid::new_v4();
        let api_key_data = ApiKeyData {
            id: key_id.to_string(),
            name: command.name,
            prefix: prefix.clone(),
            key_hash,
            scopes: command.scopes,
            user_id: UserId::parse(&command.user_id).map_err(|_| RepositoryError::DatabaseError("Invalid user_id".to_string()))?,
            tenant_id: command.tenant_id,
            expires_at: command.expires_at,
            created_at: Utc::now(),
            revoked_at: None,
        };

        self.api_key_repo.save(&api_key_data).await?;

        Ok(CreateAPIKeyResponse {
            key_id: key_id.to_string(),
            api_key: raw_key,
            prefix,
        })
    }
}
