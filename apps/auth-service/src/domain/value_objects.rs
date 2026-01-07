//! Value Objects - 值对象
//!
//! 值对象是不可变的，通过其属性值来标识。

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

// ============================================================================
// ID 类型宏
// ============================================================================

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }

            pub fn into_uuid(self) -> Uuid {
                self.0
            }

            pub fn parse(s: &str) -> Result<Self, uuid::Error> {
                Ok(Self(Uuid::parse_str(s)?))
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<Uuid> for $name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = uuid::Error;
            fn try_from(s: &str) -> Result<Self, Self::Error> {
                Ok(Self(Uuid::parse_str(s)?))
            }
        }

        impl TryFrom<String> for $name {
            type Error = uuid::Error;
            fn try_from(s: String) -> Result<Self, Self::Error> {
                Self::try_from(s.as_str())
            }
        }
    };
}

// 定义各种 ID 类型
id_type!(UserId);
id_type!(RoleId);
id_type!(PermissionId);
id_type!(RefreshTokenId);

// ============================================================================
// Permission 值对象
// ============================================================================

/// 权限值对象
/// 
/// 表示一个原子性的权限，格式为 `resource:action`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    resource: String,
    action: String,
}

impl Permission {
    pub fn new(resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            resource: resource.into().to_lowercase(),
            action: action.into().to_lowercase(),
        }
    }

    pub fn resource(&self) -> &str {
        &self.resource
    }

    pub fn action(&self) -> &str {
        &self.action
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.resource, self.action)
    }
}

// ============================================================================
// Email 值对象
// ============================================================================

/// 邮箱值对象
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    pub fn new(email: impl Into<String>) -> Result<Self, &'static str> {
        let email = email.into();
        if email.contains('@') && email.len() > 3 {
            Ok(Self(email.to_lowercase()))
        } else {
            Err("Invalid email format")
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_id_creation() {
        let id1 = UserId::new();
        let id2 = UserId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_permission_display() {
        let perm = Permission::new("sales_order", "create");
        assert_eq!(perm.to_string(), "sales_order:create");
    }

    #[test]
    fn test_email_validation() {
        assert!(Email::new("test@example.com").is_ok());
        assert!(Email::new("invalid").is_err());
    }
}
