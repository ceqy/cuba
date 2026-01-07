//! Domain Events - 领域事件
//!
//! 领域事件表示领域中发生过的重要事情，使用过去时态命名。

use crate::domain::value_objects::{Permission, RoleId, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;
use uuid::Uuid;

// ============================================================================
// DomainEvent Trait
// ============================================================================

/// 领域事件 trait
pub trait DomainEvent: Send + Sync {
    /// 返回事件名称
    fn event_name(&self) -> &'static str;

    /// 返回关联的聚合 ID
    fn aggregate_id(&self) -> Uuid;

    /// 返回事件发生时间
    fn occurred_on(&self) -> DateTime<Utc>;

    /// 用于向下转型
    fn as_any(&self) -> &dyn Any;
}

// ============================================================================
// User Events
// ============================================================================

/// 用户注册事件
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserRegisteredEvent {
    pub user_id: UserId,
    pub username: String,
    pub email: String,
    pub occurred_on: DateTime<Utc>,
}

impl UserRegisteredEvent {
    pub fn new(user_id: UserId, username: String, email: String) -> Self {
        Self {
            user_id,
            username,
            email,
            occurred_on: Utc::now(),
        }
    }
}

impl DomainEvent for UserRegisteredEvent {
    fn event_name(&self) -> &'static str {
        "UserRegistered"
    }

    fn aggregate_id(&self) -> Uuid {
        *self.user_id.as_uuid()
    }

    fn occurred_on(&self) -> DateTime<Utc> {
        self.occurred_on
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// 用户登录事件
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserLoggedInEvent {
    pub user_id: UserId,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub occurred_on: DateTime<Utc>,
}

impl UserLoggedInEvent {
    pub fn new(user_id: UserId, ip_address: Option<String>, user_agent: Option<String>) -> Self {
        Self {
            user_id,
            ip_address,
            user_agent,
            occurred_on: Utc::now(),
        }
    }
}

impl DomainEvent for UserLoggedInEvent {
    fn event_name(&self) -> &'static str {
        "UserLoggedIn"
    }

    fn aggregate_id(&self) -> Uuid {
        *self.user_id.as_uuid()
    }

    fn occurred_on(&self) -> DateTime<Utc> {
        self.occurred_on
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// 角色分配给用户事件
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoleAssignedToUserEvent {
    pub user_id: UserId,
    pub role_id: RoleId,
    pub occurred_on: DateTime<Utc>,
}

impl RoleAssignedToUserEvent {
    pub fn new(user_id: UserId, role_id: RoleId) -> Self {
        Self {
            user_id,
            role_id,
            occurred_on: Utc::now(),
        }
    }
}

impl DomainEvent for RoleAssignedToUserEvent {
    fn event_name(&self) -> &'static str {
        "RoleAssignedToUser"
    }

    fn aggregate_id(&self) -> Uuid {
        *self.user_id.as_uuid()
    }

    fn occurred_on(&self) -> DateTime<Utc> {
        self.occurred_on
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// Role Events
// ============================================================================

/// 角色创建事件
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoleCreatedEvent {
    pub role_id: RoleId,
    pub role_name: String,
    pub occurred_on: DateTime<Utc>,
}

impl RoleCreatedEvent {
    pub fn new(role_id: RoleId, role_name: String) -> Self {
        Self {
            role_id,
            role_name,
            occurred_on: Utc::now(),
        }
    }
}

impl DomainEvent for RoleCreatedEvent {
    fn event_name(&self) -> &'static str {
        "RoleCreated"
    }

    fn aggregate_id(&self) -> Uuid {
        *self.role_id.as_uuid()
    }

    fn occurred_on(&self) -> DateTime<Utc> {
        self.occurred_on
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// 权限添加到角色事件
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PermissionAddedToRoleEvent {
    pub role_id: RoleId,
    pub permission: Permission,
    pub occurred_on: DateTime<Utc>,
}

impl PermissionAddedToRoleEvent {
    pub fn new(role_id: RoleId, permission: Permission) -> Self {
        Self {
            role_id,
            permission,
            occurred_on: Utc::now(),
        }
    }
}

impl DomainEvent for PermissionAddedToRoleEvent {
    fn event_name(&self) -> &'static str {
        "PermissionAddedToRole"
    }

    fn aggregate_id(&self) -> Uuid {
        *self.role_id.as_uuid()
    }

    fn occurred_on(&self) -> DateTime<Utc> {
        self.occurred_on
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// 密码更改事件
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasswordChangedEvent {
    pub user_id: UserId,
    pub occurred_on: DateTime<Utc>,
}

impl PasswordChangedEvent {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            occurred_on: Utc::now(),
        }
    }
}

impl DomainEvent for PasswordChangedEvent {
    fn event_name(&self) -> &'static str {
        "PasswordChanged"
    }

    fn aggregate_id(&self) -> Uuid {
        *self.user_id.as_uuid()
    }

    fn occurred_on(&self) -> DateTime<Utc> {
        self.occurred_on
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
