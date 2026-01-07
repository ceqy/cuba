//! Data Transfer Objects (DTOs)
//!
//! 用于应用层和接口层之间传递数据。

use crate::domain::aggregates::User;
use serde::{Deserialize, Serialize};

/// 用户 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDto {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: String,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl UserDto {
    pub fn from_user(user: &User) -> Self {
        Self {
            user_id: user.id().to_string(),
            username: user.username().to_string(),
            email: user.email().to_string(),
            display_name: user.display_name().to_string(),
            avatar_url: user.avatar_url().to_string(),
            roles: user.roles().iter().map(|r| r.to_string()).collect(),
            is_active: user.is_active(),
            email_verified: user.email_verified(),
            created_at: user.created_at(),
            updated_at: user.updated_at(),
            last_login_at: user.last_login_at(),
        }
    }
}

/// 登录响应 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponseDto {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    pub user: UserDto,
    pub requires_2fa: bool,
    pub temp_token: Option<String>,
}

/// Token 验证结果 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenValidationDto {
    pub valid: bool,
    pub user_id: Option<String>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

/// 角色 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleDto {
    pub role_id: String,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl RoleDto {
    pub fn from_role(role: &crate::domain::aggregates::Role) -> Self {
        Self {
            role_id: role.id().to_string(),
            name: role.name().to_string(),
            description: role.description().map(|s| s.to_string()),
            permissions: role.permissions().iter().map(|p| p.to_string()).collect(),
            created_at: role.created_at(),
            updated_at: role.updated_at(),
        }
    }
}

/// 权限 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionDto {
    pub permission_id: String,
    pub resource: String,
    pub action: String,
    pub description: Option<String>,
}
