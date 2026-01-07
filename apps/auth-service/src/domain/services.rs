//! Domain Services
//!
//! 定义领域服务接口。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ============================================================================
// Token Claims & Errors
// ============================================================================

/// Access Token Claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub sub: String,        // user_id
    pub tid: String,        // tenant_id
    pub iss: String,        // issuer
    pub exp: i64,           // expiration time
    pub iat: i64,           // issued at
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub token_type: String, // "access" or "refresh"
}

/// Refresh Token Claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshTokenClaims {
    pub sub: String,        // user_id
    pub exp: i64,
    pub iat: i64,
    pub token_type: String, // "refresh"
    pub jti: String,        // unique token id
}

#[derive(Error, Debug)]
pub enum TokenError {
    #[error("Token has expired")]
    Expired,

    #[error("Invalid token")]
    Invalid,

    #[error("Token type mismatch")]
    TypeMismatch,

    #[error("Token revoked")]
    Revoked,

    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("Decoding error: {0}")]
    DecodingError(String),

    #[error("Infrastructure error: {0}")]
    InfrastructureError(String),
}

// ============================================================================
// TokenService Trait
// ============================================================================

/// Token 服务接口
#[async_trait]
pub trait TokenService: Send + Sync {
    /// 生成 Access Token 和 Refresh Token
    async fn generate_tokens(
        &self,
        user_id: String,
        tenant_id: String,
        roles: Vec<String>,
        permissions: Vec<String>,
    ) -> Result<(String, String, i64), TokenError>;

    /// 验证 Access Token
    fn validate_token(&self, token: &str) -> Result<TokenClaims, TokenError>;

    /// 刷新 Token
    async fn refresh_tokens(&self, refresh_token: &str) -> Result<(String, String, i64), TokenError>;
    
    /// 撤销单个 Refresh Token
    async fn revoke_token(&self, refresh_token: &str) -> Result<(), TokenError>;

    /// 撤销所有 Refresh Token (用于登出或安全事件)
    async fn revoke_all_for_user(&self, user_id: &String) -> Result<(), TokenError>;
}
