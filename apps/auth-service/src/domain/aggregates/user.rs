//! User Aggregate Root
//!
//! 用户聚合根，管理用户的核心信息、密码和角色分配。

use crate::domain::errors::DomainError;
use crate::domain::events::{DomainEvent, RoleAssignedToUserEvent, UserRegisteredEvent};
use crate::domain::value_objects::{Email, RoleId, UserId};
use chrono::{DateTime, Utc};

/// 用户聚合根
/// 用户聚合根
pub struct User {
    id: UserId,
    username: String,
    email: Email,
    password_hash: String,
    display_name: String,
    avatar_url: String,
    roles: Vec<RoleId>,
    is_active: bool,
    email_verified: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    last_login_at: Option<DateTime<Utc>>,
    tfa_secret: Option<String>,
    tfa_enabled: bool,
    tfa_recovery_codes: Vec<String>,
    events: Vec<Box<dyn DomainEvent>>,
}

impl Clone for User {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            username: self.username.clone(),
            email: self.email.clone(),
            password_hash: self.password_hash.clone(),
            display_name: self.display_name.clone(),
            avatar_url: self.avatar_url.clone(),
            roles: self.roles.clone(),
            is_active: self.is_active,
            email_verified: self.email_verified,
            created_at: self.created_at,
            updated_at: self.updated_at,
            last_login_at: self.last_login_at,
            tfa_secret: self.tfa_secret.clone(),
            tfa_enabled: self.tfa_enabled,
            tfa_recovery_codes: self.tfa_recovery_codes.clone(),
            events: Vec::new(),
        }
    }
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("email", &self.email)
            .field("is_active", &self.is_active)
            .finish()
    }
}

impl User {
    // ========================================================================
    // Factory Methods
    // ========================================================================

    /// 注册新用户
    ///
    /// 这是创建用户的唯一方式，确保所有业务规则都被验证。
    pub fn register(
        username: String,
        email: String,
        password: &str,
    ) -> Result<Self, DomainError> {
        // 验证用户名
        if username.is_empty() {
            return Err(DomainError::UsernameRequired);
        }
        if username.len() < 3 {
            return Err(DomainError::InvalidInput(
                "Username must be at least 3 characters".to_string(),
            ));
        }

        // 验证邮箱
        let email = Email::new(&email).map_err(|_| DomainError::InvalidEmailFormat)?;

        // 验证密码强度
        Self::validate_password(password)?;

        // 哈希密码
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|_| DomainError::InvalidInput("Failed to hash password".to_string()))?;

        let now = Utc::now();
        let user_id = UserId::new();

        let mut user = Self {
            id: user_id,
            username: username.clone(),
            email: email.clone(),
            password_hash,
            display_name: String::new(),
            avatar_url: String::new(),
            roles: Vec::new(),
            is_active: true,
            email_verified: false,
            created_at: now,
            updated_at: now,
            last_login_at: None,
            tfa_secret: None,
            tfa_enabled: false,
            tfa_recovery_codes: Vec::new(),
            events: Vec::new(),
        };

        // 发布领域事件
        user.add_event(UserRegisteredEvent::new(
            user_id,
            username,
            email.to_string(),
        ));

        Ok(user)
    }

    /// 从数据库重建用户（不触发事件）
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: UserId,
        username: String,
        email: Email,
        password_hash: String,
        display_name: String,
        avatar_url: String,
        roles: Vec<RoleId>,
        is_active: bool,
        email_verified: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        last_login_at: Option<DateTime<Utc>>,
        tfa_secret: Option<String>,
        tfa_enabled: bool,
        tfa_recovery_codes: Vec<String>,
    ) -> Self {
        Self {
            id,
            username,
            email,
            password_hash,
            display_name,
            avatar_url,
            roles,
            is_active,
            email_verified,
            created_at,
            updated_at,
            last_login_at,
            tfa_secret,
            tfa_enabled,
            tfa_recovery_codes,
            events: Vec::new(),
        }
    }

    // ========================================================================
    // Business Methods
    // ========================================================================

    /// 验证密码
    pub fn check_password(&self, password_to_check: &str) -> bool {
        bcrypt::verify(password_to_check, &self.password_hash).unwrap_or(false)
    }

    /// 分配角色
    pub fn assign_role(&mut self, role_id: RoleId) -> Result<(), DomainError> {
        if self.roles.contains(&role_id) {
            return Err(DomainError::RoleAlreadyAssigned(role_id.to_string()));
        }
        self.roles.push(role_id);
        self.updated_at = Utc::now();
        self.add_event(RoleAssignedToUserEvent::new(self.id, role_id));
        Ok(())
    }

    /// 修改密码
    pub fn update_password(&mut self, new_password: &str) -> Result<(), DomainError> {
        Self::validate_password(new_password)?;
        
        let password_hash = bcrypt::hash(new_password, bcrypt::DEFAULT_COST)
            .map_err(|_| DomainError::InvalidInput("Failed to hash password".to_string()))?;
            
        self.password_hash = password_hash;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 更新个人资料
    pub fn update_profile(
        &mut self,
        display_name: Option<String>,
        avatar_url: Option<String>,
    ) {
        if let Some(name) = display_name {
            self.display_name = name;
        }
        if let Some(url) = avatar_url {
            self.avatar_url = url;
        }
        self.updated_at = Utc::now();
    }

    /// 移除角色
    pub fn remove_role(&mut self, role_id: &RoleId) -> Result<(), DomainError> {
        if let Some(pos) = self.roles.iter().position(|r| r == role_id) {
            self.roles.remove(pos);
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(DomainError::RoleNotFound(role_id.to_string()))
        }
    }

    /// 更新最后登录时间
    pub fn record_login(&mut self) {
        self.last_login_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// 验证邮箱
    pub fn verify_email(&mut self) {
        self.email_verified = true;
        self.updated_at = Utc::now();
    }

    /// 禁用账户
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    /// 启用账户
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    /// 设置 2FA 密钥
    pub fn setup_tfa(&mut self, secret: String) {
        self.tfa_secret = Some(secret);
        self.updated_at = Utc::now();
    }

    /// 确认并启用 2FA
    pub fn enable_tfa(&mut self, recovery_codes: Vec<String>) {
        self.tfa_enabled = true;
        self.tfa_recovery_codes = recovery_codes;
        self.updated_at = Utc::now();
    }

    /// 禁用 2FA
    pub fn disable_tfa(&mut self) {
        self.tfa_enabled = false;
        self.tfa_secret = None;
        self.tfa_recovery_codes = Vec::new();
        self.updated_at = Utc::now();
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    fn validate_password(password: &str) -> Result<(), DomainError> {
        if password.len() < 8 {
            return Err(DomainError::PasswordTooShort);
        }
        if !password.chars().any(|c| c.is_uppercase()) {
            return Err(DomainError::PasswordNoUppercase);
        }
        if !password.chars().any(|c| c.is_ascii_digit()) {
            return Err(DomainError::PasswordNoDigit);
        }
        Ok(())
    }

    fn add_event(&mut self, event: impl DomainEvent + 'static) {
        self.events.push(Box::new(event));
    }

    /// 获取并清空待发布的领域事件
    pub fn drain_events(&mut self) -> Vec<Box<dyn DomainEvent>> {
        std::mem::take(&mut self.events)
    }

    // ========================================================================
    // Getters
    // ========================================================================

    pub fn id(&self) -> &UserId {
        &self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn avatar_url(&self) -> &str {
        &self.avatar_url
    }

    pub fn roles(&self) -> &[RoleId] {
        &self.roles
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn email_verified(&self) -> bool {
        self.email_verified
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn last_login_at(&self) -> Option<DateTime<Utc>> {
        self.last_login_at
    }

    pub fn tfa_secret(&self) -> Option<&str> {
        self.tfa_secret.as_deref()
    }

    pub fn tfa_enabled(&self) -> bool {
        self.tfa_enabled
    }

    pub fn tfa_recovery_codes(&self) -> &[String] {
        &self.tfa_recovery_codes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_registration() {
        let user = User::register(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "Password123",
        );
        assert!(user.is_ok());
        let user = user.unwrap();
        assert_eq!(user.username(), "testuser");
        assert!(user.is_active());
        assert!(!user.email_verified());
    }

    #[test]
    fn test_password_validation() {
        // Too short
        let result = User::register("user".to_string(), "a@b.com".to_string(), "Pass1");
        assert!(matches!(result, Err(DomainError::PasswordTooShort)));

        // No uppercase
        let result = User::register("user".to_string(), "a@b.com".to_string(), "password123");
        assert!(matches!(result, Err(DomainError::PasswordNoUppercase)));

        // No digit
        let result = User::register("user".to_string(), "a@b.com".to_string(), "PasswordABC");
        assert!(matches!(result, Err(DomainError::PasswordNoDigit)));
    }

    #[test]
    fn test_password_check() {
        let user = User::register(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "Password123",
        )
        .unwrap();
        assert!(user.check_password("Password123"));
        assert!(!user.check_password("wrongpassword"));
    }

    #[test]
    fn test_assign_role() {
        let mut user = User::register(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "Password123",
        )
        .unwrap();

        let role_id = RoleId::new();
        assert!(user.assign_role(role_id).is_ok());
        assert_eq!(user.roles().len(), 1);

        // 重复分配应该失败
        assert!(user.assign_role(role_id).is_err());
    }
}
