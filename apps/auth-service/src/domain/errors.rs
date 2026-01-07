//! Domain Errors - 领域错误
//!
//! 定义领域层可能发生的所有业务错误。

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DomainError {
    // 用户相关错误
    #[error("Username is required")]
    UsernameRequired,

    #[error("Username '{0}' is already taken")]
    UsernameAlreadyExists(String),

    #[error("Invalid email format")]
    InvalidEmailFormat,

    #[error("Email '{0}' is already registered")]
    EmailAlreadyExists(String),

    #[error("Password must be at least 8 characters long")]
    PasswordTooShort,

    #[error("Password must contain at least one uppercase letter")]
    PasswordNoUppercase,

    #[error("Password must contain at least one digit")]
    PasswordNoDigit,

    #[error("Invalid credentials")]
    InvalidCredentials,

    // 角色相关错误
    #[error("Role name is required")]
    RoleNameRequired,

    #[error("Role '{0}' already assigned to user")]
    RoleAlreadyAssigned(String),

    #[error("Role '{0}' not found")]
    RoleNotFound(String),

    // 权限相关错误
    #[error("Permission '{0}' already exists in role")]
    PermissionAlreadyExists(String),

    #[error("Permission '{0}' not found")]
    PermissionNotFound(String),

    // Token 相关错误
    #[error("Token has expired")]
    TokenExpired,

    #[error("Token is invalid")]
    TokenInvalid,

    #[error("Refresh token has been revoked")]
    RefreshTokenRevoked,

    #[error("Token has been revoked")]
    TokenRevoked,

    // 通用错误
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("User not found")]
    UserNotFound,

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Infrastructure error: {0}")]
    InfrastructureError(String),

    #[error("Operation not permitted")]
    NotPermitted,

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
}
