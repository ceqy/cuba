use crate::domain::errors::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Effect {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statement {
    pub sid: String,
    pub effect: Effect,
    pub actions: Vec<String>,
    pub resources: Vec<String>,
    // 可以在此添加 conditions 字段支持更复杂的条件
}

/// 策略聚合根
///
/// 用于定义细粒度的访问控制策略 (IAM Policy 风格)。
#[derive(Debug, Clone)]
pub struct Policy {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String, // 策略版本，如 "2024-01-01"
    pub statements: Vec<Statement>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tenant_id: String,
}

impl Policy {
    pub fn new(
        name: String,
        description: Option<String>,
        statements: Vec<Statement>,
        tenant_id: String,
    ) -> Result<Self, DomainError> {
        if name.is_empty() {
            return Err(DomainError::InvalidInput("Policy name cannot be empty".to_string()));
        }

        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            version: "1.0".to_string(),
            statements,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tenant_id,
        })
    }

    pub fn update(&mut self, statements: Vec<Statement>) {
        self.statements = statements;
        self.updated_at = Utc::now();
    }
}
