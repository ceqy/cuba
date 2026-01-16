// 科目验证领域服务
use chrono::NaiveDate;

use super::{AccountStatus, GlAccount};

/// 科目验证结果
#[derive(Debug, Clone)]
pub struct AccountValidationResult {
    pub is_valid: bool,
    pub exists: bool,
    pub is_active: bool,
    pub is_postable: bool,
    pub error_message: Option<String>,
}

impl AccountValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            exists: true,
            is_active: true,
            is_postable: true,
            error_message: None,
        }
    }

    pub fn not_found() -> Self {
        Self {
            is_valid: false,
            exists: false,
            is_active: false,
            is_postable: false,
            error_message: Some("科目不存在".to_string()),
        }
    }

    pub fn invalid(reason: String) -> Self {
        Self {
            is_valid: false,
            exists: true,
            is_active: false,
            is_postable: false,
            error_message: Some(reason),
        }
    }
}

/// 科目验证服务
pub struct AccountValidationService;

impl AccountValidationService {
    /// 验证科目是否可用于过账
    pub fn validate_for_posting(
        account: &GlAccount,
        posting_date: NaiveDate,
    ) -> AccountValidationResult {
        // 检查科目状态
        if account.status != AccountStatus::Active {
            return AccountValidationResult::invalid("科目未激活".to_string());
        }

        // 检查是否可过账
        if !account.is_postable {
            return AccountValidationResult::invalid("科目不可过账（非末级科目）".to_string());
        }

        // 检查有效期
        if let Some(valid_from) = account.valid_from {
            if posting_date < valid_from {
                return AccountValidationResult::invalid("科目尚未生效".to_string());
            }
        }

        if let Some(valid_to) = account.valid_to {
            if posting_date > valid_to {
                return AccountValidationResult::invalid("科目已过期".to_string());
            }
        }

        AccountValidationResult::valid()
    }

    /// 验证科目代码格式
    pub fn validate_account_code(account_code: &str) -> Result<(), String> {
        if account_code.is_empty() {
            return Err("科目代码不能为空".to_string());
        }

        if account_code.len() > 10 {
            return Err("科目代码长度不能超过10位".to_string());
        }

        if !account_code.chars().all(|c| c.is_ascii_digit()) {
            return Err("科目代码只能包含数字".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AccountNature, GlAccount};
    use chrono::Utc;

    #[test]
    fn test_validate_account_code() {
        assert!(AccountValidationService::validate_account_code("1001000000").is_ok());
        assert!(AccountValidationService::validate_account_code("").is_err());
        assert!(AccountValidationService::validate_account_code("12345678901").is_err());
        assert!(AccountValidationService::validate_account_code("100100ABC0").is_err());
    }

    #[test]
    fn test_validate_for_posting() {
        let account = GlAccount::new(
            "CN01".to_string(),
            "1001000000".to_string(),
            "库存现金".to_string(),
            AccountNature::Asset,
            "BALANCE_SHEET".to_string(),
        );

        let result = AccountValidationService::validate_for_posting(
            &account,
            Utc::now().naive_local().date(),
        );

        assert!(result.is_valid);
    }
}
