//! Role Aggregate Root
//!
//! 角色聚合根，管理角色及其关联的权限。

use crate::domain::errors::DomainError;
use crate::domain::events::{DomainEvent, PermissionAddedToRoleEvent, RoleCreatedEvent};
use crate::domain::value_objects::{Permission, RoleId};
use chrono::{DateTime, Utc};

/// 角色聚合根
pub struct Role {
    id: RoleId,
    name: String,
    description: Option<String>,
    permissions: Vec<Permission>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    events: Vec<Box<dyn DomainEvent>>,
}

impl std::fmt::Debug for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Role")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("description", &self.description)
            .finish()
    }
}

impl Role {
    // ========================================================================
    // Factory Methods
    // ========================================================================

    /// 创建新角色
    pub fn create(name: String, description: Option<String>) -> Result<Self, DomainError> {
        if name.is_empty() {
            return Err(DomainError::RoleNameRequired);
        }

        let now = Utc::now();
        let role_id = RoleId::new();

        let mut role = Self {
            id: role_id,
            name: name.clone(),
            description,
            permissions: Vec::new(),
            created_at: now,
            updated_at: now,
            events: Vec::new(),
        };

        role.add_event(RoleCreatedEvent::new(role_id, name));

        Ok(role)
    }

    /// 从数据库重建角色（不触发事件）
    pub fn reconstitute(
        id: RoleId,
        name: String,
        description: Option<String>,
        permissions: Vec<Permission>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            permissions,
            created_at,
            updated_at,
            events: Vec::new(),
        }
    }

    // ========================================================================
    // Business Methods
    // ========================================================================

    /// 添加权限
    pub fn add_permission(&mut self, permission: Permission) -> Result<(), DomainError> {
        if self.permissions.contains(&permission) {
            return Err(DomainError::PermissionAlreadyExists(permission.to_string()));
        }
        self.permissions.push(permission.clone());
        self.updated_at = Utc::now();
        self.add_event(PermissionAddedToRoleEvent::new(self.id, permission));
        Ok(())
    }

    /// 移除权限
    pub fn remove_permission(&mut self, permission: &Permission) -> Result<(), DomainError> {
        if let Some(pos) = self.permissions.iter().position(|p| p == permission) {
            self.permissions.remove(pos);
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(DomainError::PermissionNotFound(permission.to_string()))
        }
    }

    /// 更新角色信息
    pub fn update(&mut self, name: Option<String>, description: Option<String>) {
        if let Some(name) = name {
            self.name = name;
        }
        if description.is_some() {
            self.description = description;
        }
        self.updated_at = Utc::now();
    }

    /// 检查是否拥有某权限
    pub fn has_permission(&self, resource: &str, action: &str) -> bool {
        self.permissions
            .iter()
            .any(|p| p.resource() == resource && p.action() == action)
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

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

    pub fn id(&self) -> &RoleId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn permissions(&self) -> &[Permission] {
        &self.permissions
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_creation() {
        let role = Role::create("admin".to_string(), Some("Administrator".to_string()));
        assert!(role.is_ok());
        let role = role.unwrap();
        assert_eq!(role.name(), "admin");
        assert_eq!(role.description(), Some("Administrator"));
    }

    #[test]
    fn test_add_permission() {
        let mut role = Role::create("admin".to_string(), None).unwrap();
        let perm = Permission::new("sales_order", "create");

        assert!(role.add_permission(perm.clone()).is_ok());
        assert_eq!(role.permissions().len(), 1);

        // 重复添加应该失败
        assert!(role.add_permission(perm).is_err());
    }

    #[test]
    fn test_has_permission() {
        let mut role = Role::create("admin".to_string(), None).unwrap();
        role.add_permission(Permission::new("sales_order", "create"))
            .unwrap();

        assert!(role.has_permission("sales_order", "create"));
        assert!(!role.has_permission("sales_order", "delete"));
    }
}
