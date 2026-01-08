//! Authorization Module for GL Service
//!
//! 权限控制服务实现

use std::collections::HashSet;
use tracing::{warn, instrument};
use uuid::Uuid;

// ============================================================================
// Authorization Context
// ============================================================================

/// 用户授权上下文
#[derive(Debug, Clone)]
pub struct AuthorizationContext {
    pub user_id: Uuid,
    pub allowed_company_codes: HashSet<String>,
    pub allowed_document_types: HashSet<String>,
    pub roles: HashSet<String>,
}

impl AuthorizationContext {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            allowed_company_codes: HashSet::new(),
            allowed_document_types: HashSet::new(),
            roles: HashSet::new(),
        }
    }

    pub fn with_company_codes(mut self, codes: Vec<String>) -> Self {
        self.allowed_company_codes = codes.into_iter().collect();
        self
    }

    pub fn with_document_types(mut self, types: Vec<String>) -> Self {
        self.allowed_document_types = types.into_iter().collect();
        self
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles.into_iter().collect();
        self
    }

    /// 是否为超级管理员
    pub fn is_admin(&self) -> bool {
        self.roles.contains("ADMIN") || self.roles.contains("GL_ADMIN")
    }
}

// ============================================================================
// Authorization Errors
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum AuthorizationError {
    #[error("User not authorized for company code: {0}")]
    CompanyCodeNotAuthorized(String),

    #[error("User not authorized for document type: {0}")]
    DocumentTypeNotAuthorized(String),

    #[error("Insufficient permissions for operation: {0}")]
    InsufficientPermissions(String),
}

// ============================================================================
// Authorization Service
// ============================================================================

pub struct AuthorizationService;

impl AuthorizationService {
    /// 检查公司代码权限
    #[instrument]
    pub fn check_company_code(
        ctx: &AuthorizationContext,
        company_code: &str,
    ) -> Result<(), AuthorizationError> {
        if ctx.is_admin() {
            return Ok(());
        }

        if ctx.allowed_company_codes.is_empty() || ctx.allowed_company_codes.contains(company_code) {
            Ok(())
        } else {
            warn!(
                user_id = %ctx.user_id,
                company_code = %company_code,
                "Company code authorization denied"
            );
            Err(AuthorizationError::CompanyCodeNotAuthorized(company_code.to_string()))
        }
    }

    /// 检查凭证类型权限
    #[instrument]
    pub fn check_document_type(
        ctx: &AuthorizationContext,
        document_type: &str,
    ) -> Result<(), AuthorizationError> {
        if ctx.is_admin() {
            return Ok(());
        }

        if ctx.allowed_document_types.is_empty() || ctx.allowed_document_types.contains(document_type) {
            Ok(())
        } else {
            warn!(
                user_id = %ctx.user_id,
                document_type = %document_type,
                "Document type authorization denied"
            );
            Err(AuthorizationError::DocumentTypeNotAuthorized(document_type.to_string()))
        }
    }

    /// 检查过账权限
    pub fn check_posting_permission(ctx: &AuthorizationContext) -> Result<(), AuthorizationError> {
        if ctx.is_admin() || ctx.roles.contains("GL_POST") {
            Ok(())
        } else {
            Err(AuthorizationError::InsufficientPermissions("POST".to_string()))
        }
    }

    /// 检查冲销权限
    pub fn check_reversal_permission(ctx: &AuthorizationContext) -> Result<(), AuthorizationError> {
        if ctx.is_admin() || ctx.roles.contains("GL_REVERSE") {
            Ok(())
        } else {
            Err(AuthorizationError::InsufficientPermissions("REVERSE".to_string()))
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_bypasses_all_checks() {
        let ctx = AuthorizationContext::new(Uuid::new_v4())
            .with_roles(vec!["ADMIN".to_string()]);

        assert!(AuthorizationService::check_company_code(&ctx, "1000").is_ok());
        assert!(AuthorizationService::check_document_type(&ctx, "SA").is_ok());
        assert!(AuthorizationService::check_posting_permission(&ctx).is_ok());
    }

    #[test]
    fn test_company_code_authorization() {
        let ctx = AuthorizationContext::new(Uuid::new_v4())
            .with_company_codes(vec!["1000".to_string(), "2000".to_string()]);

        assert!(AuthorizationService::check_company_code(&ctx, "1000").is_ok());
        assert!(AuthorizationService::check_company_code(&ctx, "3000").is_err());
    }
}
