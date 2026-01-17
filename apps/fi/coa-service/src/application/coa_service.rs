// COA Application Service - COA 应用服务
use chrono::NaiveDate;
use std::sync::Arc;

use crate::domain::{AccountNature, AccountValidationResult, GlAccount};
use crate::infrastructure::GlAccountRepository;

/// COA 应用服务
pub struct CoaApplicationService {
    repository: Arc<dyn GlAccountRepository>,
}

impl CoaApplicationService {
    pub fn new(repository: Arc<dyn GlAccountRepository>) -> Self {
        Self { repository }
    }

    /// 创建科目
    pub async fn create_account(
        &self,
        account: GlAccount,
    ) -> anyhow::Result<String> {
        // 验证科目代码格式
        account.validate_account_code().map_err(anyhow::Error::msg)?;

        // 检查科目是否已存在
        if let Some(_existing) = self
            .repository
            .find_by_code(&account.chart_code, &account.account_code)
            .await?
        {
            return Err(anyhow::anyhow!("科目代码已存在"));
        }

        // 保存科目
        self.repository.create(&account).await?;

        Ok(account.account_code)
    }

    /// 获取科目详情
    pub async fn get_account(
        &self,
        chart_code: &str,
        account_code: &str,
    ) -> anyhow::Result<Option<GlAccount>> {
        self.repository.find_by_code(chart_code, account_code).await
    }

    /// 更新科目
    pub async fn update_account(
        &self,
        account: GlAccount,
    ) -> anyhow::Result<()> {
        // 检查科目是否存在
        if self
            .repository
            .find_by_code(&account.chart_code, &account.account_code)
            .await?
            .is_none()
        {
            return Err(anyhow::anyhow!("科目不存在"));
        }

        self.repository.update(&account).await
    }

    /// 删除科目
    pub async fn delete_account(
        &self,
        chart_code: &str,
        account_code: &str,
        soft_delete: bool,
    ) -> anyhow::Result<()> {
        if soft_delete {
            // 软删除：标记为删除状态
            if let Some(mut account) = self.repository.find_by_code(chart_code, account_code).await? {
                account.deactivate();
                self.repository.update(&account).await?;
            }
        } else {
            // 物理删除
            self.repository.delete(chart_code, account_code).await?;
        }
        Ok(())
    }

    /// 查询科目列表
    pub async fn list_accounts(
        &self,
        chart_code: &str,
    ) -> anyhow::Result<Vec<GlAccount>> {
        self.repository.find_all(chart_code).await
    }

    /// 按科目性质查询
    pub async fn list_accounts_by_nature(
        &self,
        chart_code: &str,
        nature: AccountNature,
    ) -> anyhow::Result<Vec<GlAccount>> {
        self.repository.find_by_nature(chart_code, &nature).await
    }

    /// 查询可过账科目
    pub async fn list_postable_accounts(
        &self,
        chart_code: &str,
    ) -> anyhow::Result<Vec<GlAccount>> {
        self.repository.find_postable_accounts(chart_code).await
    }

    /// 验证科目
    pub async fn validate_account(
        &self,
        chart_code: &str,
        account_code: &str,
        posting_date: NaiveDate,
    ) -> anyhow::Result<AccountValidationResult> {
        self.repository
            .validate_account(chart_code, account_code, posting_date)
            .await
    }

    /// 批量验证科目
    pub async fn batch_validate_accounts(
        &self,
        chart_code: &str,
        account_codes: Vec<String>,
        posting_date: NaiveDate,
    ) -> anyhow::Result<Vec<AccountValidationResult>> {
        let mut results = Vec::new();
        for account_code in account_codes {
            let result = self
                .validate_account(chart_code, &account_code, posting_date)
                .await?;
            results.push(result);
        }
        Ok(results)
    }

    /// 查询子科目列表
    pub async fn list_child_accounts(
        &self,
        chart_code: &str,
        parent_account: &str,
    ) -> anyhow::Result<Vec<GlAccount>> {
        self.repository.find_children(chart_code, parent_account).await
    }

    /// 查询科目路径（从根到当前科目的所有祖先）
    pub async fn get_account_path(
        &self,
        chart_code: &str,
        account_code: &str,
    ) -> anyhow::Result<Vec<GlAccount>> {
        self.repository.find_ancestors(chart_code, account_code).await
    }

    /// 批量创建科目
    pub async fn batch_create_accounts(
        &self,
        accounts: Vec<GlAccount>,
    ) -> anyhow::Result<Vec<String>> {
        // 验证所有科目代码
        for account in &accounts {
            account.validate_account_code().map_err(anyhow::Error::msg)?;
        }
        self.repository.batch_create(&accounts).await
    }
}
