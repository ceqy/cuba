// COA Service Client - COA 服务客户端
use tonic::transport::Channel;
use chrono::NaiveDate;

// Include generated COA proto code
pub mod coa_proto {
    tonic::include_proto!("fi.coa.v1");
}

use coa_proto::{
    chart_of_accounts_service_client::ChartOfAccountsServiceClient,
    ValidateGlAccountRequest, ValidateGlAccountResponse,
    BatchValidateGlAccountsRequest, BatchValidateGlAccountsResponse,
};

/// COA 服务客户端
#[derive(Clone)]
pub struct CoaClient {
    client: ChartOfAccountsServiceClient<Channel>,
}

impl CoaClient {
    /// 连接到 COA 服务
    pub async fn connect(endpoint: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let client = ChartOfAccountsServiceClient::connect(endpoint.to_string()).await?;
        Ok(Self { client })
    }

    /// 验证单个科目
    pub async fn validate_account(
        &mut self,
        chart_code: &str,
        account_code: &str,
        company_code: Option<&str>,
        posting_date: NaiveDate,
    ) -> Result<AccountValidationResult, Box<dyn std::error::Error + Send + Sync>> {
        let request = ValidateGlAccountRequest {
            chart_of_accounts: chart_code.to_string(),
            account_code: account_code.to_string(),
            company_code: company_code.unwrap_or("").to_string(),
            check_postable: true,
            posting_date: Some(prost_types::Timestamp {
                seconds: posting_date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
                nanos: 0,
            }),
            context: None,
        };

        let response = self.client.validate_gl_account(request).await?;
        let result = response.into_inner().result.ok_or("No validation result")?;

        Ok(AccountValidationResult {
            account_code: result.account_code,
            is_valid: result.is_valid,
            exists: result.exists,
            is_active: result.is_active,
            is_postable: result.is_postable,
            error_messages: result.messages.into_iter().map(|m| m.message).collect(),
        })
    }

    /// 批量验证科目
    pub async fn batch_validate_accounts(
        &mut self,
        chart_code: &str,
        account_codes: Vec<String>,
        company_code: Option<&str>,
    ) -> Result<Vec<AccountValidationResult>, Box<dyn std::error::Error + Send + Sync>> {
        let request = BatchValidateGlAccountsRequest {
            chart_of_accounts: chart_code.to_string(),
            account_codes,
            company_code: company_code.unwrap_or("").to_string(),
            context: None,
        };

        let response = self.client.batch_validate_gl_accounts(request).await?;
        let inner = response.into_inner();

        Ok(inner
            .results
            .into_iter()
            .map(|r| AccountValidationResult {
                account_code: r.account_code,
                is_valid: r.is_valid,
                exists: r.exists,
                is_active: r.is_active,
                is_postable: r.is_postable,
                error_messages: r.messages.into_iter().map(|m| m.message).collect(),
            })
            .collect())
    }
}

/// 科目验证结果
#[derive(Debug, Clone)]
pub struct AccountValidationResult {
    pub account_code: String,
    pub is_valid: bool,
    pub exists: bool,
    pub is_active: bool,
    pub is_postable: bool,
    pub error_messages: Vec<String>,
}

impl AccountValidationResult {
    /// 获取验证失败的原因
    pub fn get_error_message(&self) -> Option<String> {
        if self.is_valid {
            None
        } else if !self.exists {
            Some(format!("科目 {} 不存在", self.account_code))
        } else if !self.is_active {
            Some(format!("科目 {} 未激活", self.account_code))
        } else if !self.is_postable {
            Some(format!("科目 {} 不可过账", self.account_code))
        } else if !self.error_messages.is_empty() {
            Some(self.error_messages.join("; "))
        } else {
            Some(format!("科目 {} 验证失败", self.account_code))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要 COA 服务运行
    async fn test_validate_account() {
        let mut client = CoaClient::connect("http://localhost:50060")
            .await
            .expect("Failed to connect to COA service");

        let result = client
            .validate_account(
                "CN01",
                "1001000000",
                Some("1000"),
                chrono::Utc::now().naive_utc().date(),
            )
            .await
            .expect("Failed to validate account");

        assert!(result.is_valid);
        assert!(result.exists);
    }
}
