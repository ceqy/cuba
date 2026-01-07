//! Repositories - 仓储接口
//!
//! 领域层只定义仓储接口，具体实现在基础设施层。

use crate::domain::aggregates::{Role, User};
use crate::domain::value_objects::{Permission, RoleId, UserId, PermissionId};
use async_trait::async_trait;
use thiserror::Error;

// ============================================================================
// Repository Errors
// ============================================================================

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Concurrency conflict for aggregate {0}")]
    ConcurrencyError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Item not found")]
    NotFound,

    #[error("Duplicate key: {0}")]
    DuplicateKey(String),
}

impl From<RepositoryError> for crate::domain::errors::DomainError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound => crate::domain::errors::DomainError::NotFound("Item not found".to_string()),
            RepositoryError::DuplicateKey(m) => crate::domain::errors::DomainError::AlreadyExists(m),
            _ => crate::domain::errors::DomainError::InfrastructureError(err.to_string()),
        }
    }
}

// ============================================================================
// User Repository
// ============================================================================

/// 用户仓储接口
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// 保存用户（新增或更新）
    async fn save(&self, user: &mut User) -> Result<(), RepositoryError>;

    /// 根据 ID 查找用户
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, RepositoryError>;

    /// 根据用户名查找用户
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError>;

    /// 根据邮箱查找用户
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError>;

    /// 检查用户名是否存在
    async fn username_exists(&self, username: &str) -> Result<bool, RepositoryError>;

    /// 检查邮箱是否存在
    async fn email_exists(&self, email: &str) -> Result<bool, RepositoryError>;

    /// 获取用户的所有权限（通过角色聚合）
    async fn get_user_permissions(&self, id: &UserId) -> Result<Vec<Permission>, RepositoryError>;

    /// 删除用户
    async fn delete(&self, id: &UserId) -> Result<(), RepositoryError>;

    /// 列出所有用户（支持搜索和分页）
    async fn find_all(
        &self,
        search: Option<&str>,
        role_id: Option<&RoleId>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<User>, RepositoryError>;

    /// 获取满足条件的用户总数
    async fn count_all(
        &self,
        search: Option<&str>,
        role_id: Option<&RoleId>,
    ) -> Result<i64, RepositoryError>;
}

// ============================================================================
// Role Repository
// ============================================================================

/// 角色仓储接口
#[async_trait]
pub trait RoleRepository: Send + Sync {
    /// 保存角色（新增或更新）
    async fn save(&self, role: &mut Role) -> Result<(), RepositoryError>;

    /// 根据 ID 查找角色
    async fn find_by_id(&self, id: &RoleId) -> Result<Option<Role>, RepositoryError>;

    /// 根据名称查找角色
    async fn find_by_name(&self, name: &str) -> Result<Option<Role>, RepositoryError>;

    /// 获取所有角色
    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<Role>, RepositoryError>;

    /// 删除角色
    async fn delete(&self, id: &RoleId) -> Result<(), RepositoryError>;

    /// 根据 ID 查找权限
    async fn find_permission_by_id(&self, id: &PermissionId) -> Result<Option<Permission>, RepositoryError>;

    /// 获取所有权限
    async fn find_all_permissions(&self, limit: i64, offset: i64) -> Result<Vec<(PermissionId, Permission)>, RepositoryError>;

    // Add Pagination Count Support
    async fn count_all(&self) -> Result<i64, RepositoryError>;
    async fn count_all_permissions(&self) -> Result<i64, RepositoryError>;
}

// ============================================================================
// Refresh Token Repository
// ============================================================================

/// Refresh Token 数据
#[derive(Debug, Clone)]
pub struct RefreshTokenData {
    pub id: String,
    pub user_id: UserId,
    pub token_hash: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub is_revoked: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Refresh Token 仓储接口
#[async_trait]
pub trait RefreshTokenRepository: Send + Sync {
    /// 保存 Refresh Token
    async fn save(&self, token: &RefreshTokenData) -> Result<(), RepositoryError>;

    /// 根据 Token Hash 查找
    async fn find_by_hash(&self, token_hash: &str) -> Result<Option<RefreshTokenData>, RepositoryError>;

    /// 撤销 Token
    async fn revoke(&self, token_hash: &str) -> Result<(), RepositoryError>;

    /// 撤销用户所有 Token
    async fn revoke_all_for_user(&self, user_id: &UserId) -> Result<(), RepositoryError>;

    /// 清理过期 Token
    async fn cleanup_expired(&self) -> Result<u64, RepositoryError>;
}

// ============================================================================
// Verification Token Repository
// ============================================================================

#[derive(Debug, Clone)]
pub enum VerificationTokenType {
    PasswordReset,
    EmailVerification,
    OAuth2Code,
}

impl ToString for VerificationTokenType {
    fn to_string(&self) -> String {
        match self {
            Self::PasswordReset => "password_reset".to_string(),
            Self::EmailVerification => "email_verification".to_string(),
            Self::OAuth2Code => "oauth2_code".to_string(),
        }
    }
}

impl From<String> for VerificationTokenType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "password_reset" => Self::PasswordReset,
            "email_verification" => Self::EmailVerification,
            "oauth2_code" => Self::OAuth2Code,
            _ => Self::EmailVerification, // Default or Error
        }
    }
}

#[derive(Debug, Clone)]
pub struct VerificationTokenData {
    pub id: String,
    pub user_id: UserId,
    pub token_type: VerificationTokenType,
    pub token_hash: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub used_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[async_trait]
pub trait VerificationRepository: Send + Sync {
    async fn save(&self, token: &VerificationTokenData) -> Result<(), RepositoryError>;
    async fn find_by_hash(&self, token_hash: &str, token_type: VerificationTokenType) -> Result<Option<VerificationTokenData>, RepositoryError>;
    async fn mark_as_used(&self, token_hash: &str) -> Result<(), RepositoryError>;
    async fn delete_expired(&self, expires_before: chrono::DateTime<chrono::Utc>) -> Result<u64, RepositoryError>;
}

// ============================================================================
// API Key Repository
// ============================================================================

#[derive(Debug, Clone)]
pub struct ApiKeyData {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub key_hash: String,
    pub scopes: Vec<String>,
    pub user_id: UserId,
    pub tenant_id: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[async_trait]
pub trait ApiKeyRepository: Send + Sync {
    async fn save(&self, key: &ApiKeyData) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<ApiKeyData>, RepositoryError>;
    async fn find_by_user(&self, user_id: &UserId, limit: i64, offset: i64) -> Result<Vec<ApiKeyData>, RepositoryError>;
    async fn count_by_user(&self, user_id: &UserId) -> Result<i64, RepositoryError>;
    async fn revoke(&self, id: &str) -> Result<(), RepositoryError>;
}

// ============================================================================
// Audit Log Repository
// ============================================================================

#[derive(Debug, Clone)]
pub struct AuditLogData {
    pub id: String,
    pub user_id: String,
    pub tenant_id: String,
    pub action: String,
    pub resource: String,
    pub ip_address: String,
    pub user_agent: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub details: std::collections::HashMap<String, String>,
}

#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    async fn save(&self, log: &AuditLogData) -> Result<(), RepositoryError>;
    async fn find_logs(
        &self,
        user_id: Option<&str>,
        tenant_id: Option<&str>,
        action: Option<&str>,
        start_time: Option<chrono::DateTime<chrono::Utc>>,
        end_time: Option<chrono::DateTime<chrono::Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLogData>, RepositoryError>;
    async fn count_logs(
        &self,
        user_id: Option<&str>,
        tenant_id: Option<&str>,
        action: Option<&str>,
        start_time: Option<chrono::DateTime<chrono::Utc>>,
        end_time: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<i64, RepositoryError>;
}

// ============================================================================
// Session Repository
// ============================================================================

#[derive(Debug, Clone)]
pub struct SessionData {
    pub session_id: String,
    pub user_id: UserId,
    pub device_name: String,
    pub ip_address: String,
    pub location: String,
    pub last_active: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn save(&self, session: &SessionData) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, session_id: &str) -> Result<Option<SessionData>, RepositoryError>;
    async fn find_by_user(&self, user_id: &UserId) -> Result<Vec<SessionData>, RepositoryError>;
    async fn delete(&self, session_id: &str) -> Result<(), RepositoryError>;
    async fn delete_all_for_user(&self, user_id: &UserId) -> Result<(), RepositoryError>;
    async fn update_last_active(&self, session_id: &str, last_active: chrono::DateTime<chrono::Utc>) -> Result<(), RepositoryError>;
    async fn count_active(&self) -> Result<i64, RepositoryError>;
}

// ============================================================================
// Client Repository (OAuth2)
// ============================================================================

#[derive(Debug, Clone)]
pub struct ClientData {
    pub client_id: String,
    pub client_secret: String, // Hashed
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub grant_types: Vec<String>,
    pub scopes: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
pub trait ClientRepository: Send + Sync {
    async fn save(&self, client: &ClientData) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, client_id: &str) -> Result<Option<ClientData>, RepositoryError>;
    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<ClientData>, RepositoryError>;
    async fn count_all(&self) -> Result<i64, RepositoryError>;
    async fn delete(&self, client_id: &str) -> Result<(), RepositoryError>;
}

pub mod policy_repository;
pub use policy_repository::PolicyRepository;
