//! Authorize Handler (OAuth2)

use crate::domain::repositories::{VerificationRepository, VerificationTokenData, VerificationTokenType, RepositoryError};
use crate::domain::value_objects::UserId;
use chrono::{Duration, Utc};
use std::sync::Arc;
use uuid::Uuid;
use ring::rand::{SecureRandom, SystemRandom};
use base64::{engine::general_purpose, Engine as _};

pub struct AuthorizeCommand {
    pub user_id: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: String,
    pub state: String,
    pub response_type: String,
}

pub struct AuthorizeResponse {
    pub code: String,
    pub state: String,
}

pub struct AuthorizeHandler {
    verification_repo: Arc<dyn VerificationRepository>,
}

impl AuthorizeHandler {
    pub fn new(verification_repo: Arc<dyn VerificationRepository>) -> Self {
        Self { verification_repo }
    }

    pub async fn handle(&self, command: AuthorizeCommand) -> Result<AuthorizeResponse, RepositoryError> {
        if command.response_type != "code" {
            return Err(RepositoryError::DatabaseError("Unsupported response type".to_string()));
        }

        // 1. 生成授权码
        let mut code_bytes = [0u8; 32];
        let rng = SystemRandom::new();
        rng.fill(&mut code_bytes).map_err(|_| RepositoryError::DatabaseError("RNG failure".to_string()))?;
        let code = general_purpose::URL_SAFE_NO_PAD.encode(code_bytes);

        // 2. 存储授权码
        let token_data = VerificationTokenData {
            id: Uuid::new_v4().to_string(),
            user_id: UserId::parse(&command.user_id).map_err(|_| RepositoryError::DatabaseError("Invalid user_id".to_string()))?,
            token_type: VerificationTokenType::OAuth2Code,
            token_hash: code.clone(), // 这里可以直接存 code 或其哈希
            expires_at: Utc::now() + Duration::minutes(10), // 授权码通常 10 分钟过期
            created_at: Utc::now(),
            used_at: None,
        };

        self.verification_repo.save(&token_data).await?;

        Ok(AuthorizeResponse {
            code,
            state: command.state,
        })
    }
}
