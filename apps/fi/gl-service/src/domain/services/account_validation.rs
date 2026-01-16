// Account validation service using COA client
use std::sync::Arc;
use chrono::NaiveDate;
use crate::infrastructure::clients::{CoaClient, AccountValidationResult};

/// 科目验证服务
pub struct AccountValidationService {
    coa_client: Arc<tokio::sync::Mutex<CoaClient>>,
    chart_code: String,
}

impl AccountValidationService {
    pub fn new(coa_client: CoaClient, chart_code: String) -> Self {
        Self {
            coa_client: Arc::new(tokio::sync::Mutex::new(coa_client)),
            chart_code,
        }
    }

    /// 验证单个科目
    pub async fn validate_account(
        &self,
        account_code: &str,
        company_code: Option<&str>,
        posting_date: NaiveDate,
    ) -> Result<AccountValidationResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut client = self.coa_client.lock().await;
        client
            .validate_account(&self.chart_code, account_code, company_code, posting_date)
            .await
    }

    /// 批量验证科目
    pub async fn batch_validate_accounts(
        &self,
        account_codes: Vec<String>,
        company_code: Option<&str>,
    ) -> Result<Vec<AccountValidationResult>, Box<dyn std::error::Error + Send + Sync>> {
        let mut client = self.coa_client.lock().await;
        client
            .batch_validate_accounts(&self.chart_code, account_codes, company_code)
            .await
    }

    /// 验证凭证中的所有科目
    pub async fn validate_journal_entry_accounts(
        &self,
        account_codes: Vec<String>,
        company_code: &str,
        posting_date: NaiveDate,
    ) -> Result<Vec<AccountValidationResult>, Box<dyn std::error::Error + Send + Sync>> {
        // 批量验证所有科目
        let results = self
            .batch_validate_accounts(account_codes.clone(), Some(company_code))
            .await?;

        // 检查是否有无效科目
        let invalid_accounts: Vec<_> = results
            .iter()
            .filter(|r| !r.is_valid)
            .collect();

        if !invalid_accounts.is_empty() {
            let error_msg = invalid_accounts
                .iter()
                .filter_map(|r| r.get_error_message())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(format!("科目验证失败: {}", error_msg).into());
        }

        Ok(results)
    }
}
