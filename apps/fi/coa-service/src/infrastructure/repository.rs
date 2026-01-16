// Repository implementation - 仓储实现
use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::{
    AccountGroup, AccountNature, AccountStatus, AccountText, AccountValidationResult,
    BalanceIndicator, CompanyCodeAccountData, GlAccount,
};

/// 科目仓储trait
#[async_trait]
pub trait GlAccountRepository: Send + Sync {
    async fn create(&self, account: &GlAccount) -> Result<(), Box<dyn std::error::Error>>;
    async fn update(&self, account: &GlAccount) -> Result<(), Box<dyn std::error::Error>>;
    async fn delete(&self, chart_code: &str, account_code: &str) -> Result<(), Box<dyn std::error::Error>>;
    async fn find_by_code(&self, chart_code: &str, account_code: &str) -> Result<Option<GlAccount>, Box<dyn std::error::Error>>;
    async fn find_all(&self, chart_code: &str) -> Result<Vec<GlAccount>, Box<dyn std::error::Error>>;
    async fn find_by_nature(&self, chart_code: &str, nature: &AccountNature) -> Result<Vec<GlAccount>, Box<dyn std::error::Error>>;
    async fn find_postable_accounts(&self, chart_code: &str) -> Result<Vec<GlAccount>, Box<dyn std::error::Error>>;
    async fn validate_account(&self, chart_code: &str, account_code: &str, posting_date: NaiveDate) -> Result<AccountValidationResult, Box<dyn std::error::Error>>;
}

/// PostgreSQL 科目仓储实现
pub struct PgGlAccountRepository {
    pool: PgPool,
}

impl PgGlAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GlAccountRepository for PgGlAccountRepository {
    async fn create(&self, account: &GlAccount) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            r#"
            INSERT INTO gl_accounts (
                id, chart_code, account_code, account_name, account_name_long,
                account_nature, account_category, account_group,
                account_level, parent_account, is_leaf_account, full_path, depth,
                is_postable, is_cost_element, line_item_display, open_item_management,
                balance_indicator, currency, only_local_currency, exchange_rate_diff,
                tax_relevant, tax_category, status, valid_from, valid_to,
                created_by, created_at, tenant_id
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13,
                $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26,
                $27, $28, $29
            )
            "#,
            account.id,
            account.chart_code,
            account.account_code,
            account.account_name,
            account.account_name_long,
            format!("{:?}", account.account_nature).to_uppercase(),
            account.account_category,
            account.account_group,
            account.account_level,
            account.parent_account,
            account.is_leaf_account,
            account.full_path,
            account.depth,
            account.is_postable,
            account.is_cost_element,
            account.line_item_display,
            account.open_item_management,
            match account.balance_indicator {
                BalanceIndicator::Debit => "D",
                BalanceIndicator::Credit => "C",
            },
            account.currency,
            account.only_local_currency,
            account.exchange_rate_diff,
            account.tax_relevant,
            account.tax_category,
            format!("{:?}", account.status).to_uppercase(),
            account.valid_from,
            account.valid_to,
            account.created_by,
            account.created_at,
            account.tenant_id,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update(&self, account: &GlAccount) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            r#"
            UPDATE gl_accounts SET
                account_name = $1,
                account_name_long = $2,
                account_nature = $3,
                account_category = $4,
                status = $5,
                changed_by = $6,
                changed_at = $7
            WHERE chart_code = $8 AND account_code = $9
            "#,
            account.account_name,
            account.account_name_long,
            format!("{:?}", account.account_nature).to_uppercase(),
            account.account_category,
            format!("{:?}", account.status).to_uppercase(),
            account.changed_by,
            account.changed_at,
            account.chart_code,
            account.account_code,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, chart_code: &str, account_code: &str) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            "DELETE FROM gl_accounts WHERE chart_code = $1 AND account_code = $2",
            chart_code,
            account_code,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_code(&self, chart_code: &str, account_code: &str) -> Result<Option<GlAccount>, Box<dyn std::error::Error>> {
        let row = sqlx::query!(
            r#"
            SELECT * FROM gl_accounts
            WHERE chart_code = $1 AND account_code = $2
            "#,
            chart_code,
            account_code,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| GlAccount {
            id: r.id,
            chart_code: r.chart_code,
            account_code: r.account_code,
            account_name: r.account_name,
            account_name_long: r.account_name_long,
            search_key: r.search_key,
            account_nature: parse_account_nature(&r.account_nature),
            account_category: r.account_category,
            account_group: r.account_group,
            account_level: r.account_level,
            parent_account: r.parent_account,
            is_leaf_account: r.is_leaf_account,
            full_path: r.full_path,
            depth: r.depth,
            is_postable: r.is_postable,
            is_cost_element: r.is_cost_element,
            line_item_display: r.line_item_display,
            open_item_management: r.open_item_management,
            balance_indicator: if r.balance_indicator.as_deref() == Some("C") {
                BalanceIndicator::Credit
            } else {
                BalanceIndicator::Debit
            },
            currency: r.currency,
            only_local_currency: r.only_local_currency,
            exchange_rate_diff: r.exchange_rate_diff,
            tax_relevant: r.tax_relevant,
            tax_category: r.tax_category,
            status: parse_account_status(&r.status),
            valid_from: r.valid_from,
            valid_to: r.valid_to,
            created_by: r.created_by,
            created_at: r.created_at.and_utc(),
            changed_by: r.changed_by,
            changed_at: r.changed_at.map(|dt| dt.and_utc()),
            tenant_id: r.tenant_id,
        }))
    }

    async fn find_all(&self, chart_code: &str) -> Result<Vec<GlAccount>, Box<dyn std::error::Error>> {
        let rows = sqlx::query!(
            "SELECT * FROM gl_accounts WHERE chart_code = $1 ORDER BY account_code",
            chart_code,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| GlAccount {
            id: r.id,
            chart_code: r.chart_code,
            account_code: r.account_code,
            account_name: r.account_name,
            account_name_long: r.account_name_long,
            search_key: r.search_key,
            account_nature: parse_account_nature(&r.account_nature),
            account_category: r.account_category,
            account_group: r.account_group,
            account_level: r.account_level,
            parent_account: r.parent_account,
            is_leaf_account: r.is_leaf_account,
            full_path: r.full_path,
            depth: r.depth,
            is_postable: r.is_postable,
            is_cost_element: r.is_cost_element,
            line_item_display: r.line_item_display,
            open_item_management: r.open_item_management,
            balance_indicator: if r.balance_indicator.as_deref() == Some("C") {
                BalanceIndicator::Credit
            } else {
                BalanceIndicator::Debit
            },
            currency: r.currency,
            only_local_currency: r.only_local_currency,
            exchange_rate_diff: r.exchange_rate_diff,
            tax_relevant: r.tax_relevant,
            tax_category: r.tax_category,
            status: parse_account_status(&r.status),
            valid_from: r.valid_from,
            valid_to: r.valid_to,
            created_by: r.created_by,
            created_at: r.created_at.and_utc(),
            changed_by: r.changed_by,
            changed_at: r.changed_at.map(|dt| dt.and_utc()),
            tenant_id: r.tenant_id,
        }).collect())
    }

    async fn find_by_nature(&self, chart_code: &str, nature: &AccountNature) -> Result<Vec<GlAccount>, Box<dyn std::error::Error>> {
        let nature_str = format!("{:?}", nature).to_uppercase();
        let rows = sqlx::query!(
            "SELECT * FROM gl_accounts WHERE chart_code = $1 AND account_nature = $2",
            chart_code,
            nature_str,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| GlAccount {
            id: r.id,
            chart_code: r.chart_code,
            account_code: r.account_code,
            account_name: r.account_name,
            account_name_long: r.account_name_long,
            search_key: r.search_key,
            account_nature: parse_account_nature(&r.account_nature),
            account_category: r.account_category,
            account_group: r.account_group,
            account_level: r.account_level,
            parent_account: r.parent_account,
            is_leaf_account: r.is_leaf_account,
            full_path: r.full_path,
            depth: r.depth,
            is_postable: r.is_postable,
            is_cost_element: r.is_cost_element,
            line_item_display: r.line_item_display,
            open_item_management: r.open_item_management,
            balance_indicator: if r.balance_indicator.as_deref() == Some("C") {
                BalanceIndicator::Credit
            } else {
                BalanceIndicator::Debit
            },
            currency: r.currency,
            only_local_currency: r.only_local_currency,
            exchange_rate_diff: r.exchange_rate_diff,
            tax_relevant: r.tax_relevant,
            tax_category: r.tax_category,
            status: parse_account_status(&r.status),
            valid_from: r.valid_from,
            valid_to: r.valid_to,
            created_by: r.created_by,
            created_at: r.created_at.and_utc(),
            changed_by: r.changed_by,
            changed_at: r.changed_at.map(|dt| dt.and_utc()),
            tenant_id: r.tenant_id,
        }).collect())
    }

    async fn find_postable_accounts(&self, chart_code: &str) -> Result<Vec<GlAccount>, Box<dyn std::error::Error>> {
        let rows = sqlx::query!(
            "SELECT * FROM vw_postable_accounts WHERE chart_code = $1",
            chart_code,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| GlAccount {
            id: Uuid::new_v4(), // 视图没有ID，临时生成
            chart_code: r.chart_code.unwrap_or_default(),
            account_code: r.account_code.unwrap_or_default(),
            account_name: r.account_name.unwrap_or_default(),
            account_name_long: None,
            search_key: None,
            account_nature: r.account_nature.as_ref().map(|s| parse_account_nature(s)).unwrap_or(AccountNature::Asset),
            account_category: String::new(),
            account_group: None,
            account_level: r.account_level.unwrap_or(1),
            parent_account: r.parent_account,
            is_leaf_account: true,
            full_path: None,
            depth: 0,
            is_postable: true,
            is_cost_element: false,
            line_item_display: true,
            open_item_management: false,
            balance_indicator: r.balance_indicator.as_deref().map(|s| if s == "C" {
                BalanceIndicator::Credit
            } else {
                BalanceIndicator::Debit
            }).unwrap_or(BalanceIndicator::Debit),
            currency: None,
            only_local_currency: true,
            exchange_rate_diff: false,
            tax_relevant: false,
            tax_category: None,
            status: r.status.as_ref().map(|s| parse_account_status(s)).unwrap_or(AccountStatus::Active),
            valid_from: None,
            valid_to: None,
            created_by: None,
            created_at: chrono::Utc::now(),
            changed_by: None,
            changed_at: None,
            tenant_id: None,
        }).collect())
    }

    async fn validate_account(&self, chart_code: &str, account_code: &str, posting_date: NaiveDate) -> Result<AccountValidationResult, Box<dyn std::error::Error>> {
        let result = sqlx::query!(
            r#"
            SELECT * FROM fn_validate_account($1, $2, $3)
            "#,
            chart_code,
            account_code,
            posting_date,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(AccountValidationResult {
            is_valid: result.is_valid.unwrap_or(false),
            exists: result.exists.unwrap_or(false),
            is_active: result.is_active.unwrap_or(false),
            is_postable: result.is_postable.unwrap_or(false),
            error_message: result.error_message,
        })
    }
}

// Helper functions
fn parse_account_nature(s: &str) -> AccountNature {
    match s.to_uppercase().as_str() {
        "ASSET" => AccountNature::Asset,
        "LIABILITY" => AccountNature::Liability,
        "EQUITY" => AccountNature::Equity,
        "REVENUE" => AccountNature::Revenue,
        "EXPENSE" => AccountNature::Expense,
        "PROFITLOSS" | "PROFIT_LOSS" => AccountNature::ProfitLoss,
        _ => AccountNature::Asset,
    }
}

fn parse_account_status(s: &str) -> AccountStatus {
    match s.to_uppercase().as_str() {
        "ACTIVE" => AccountStatus::Active,
        "INACTIVE" => AccountStatus::Inactive,
        "BLOCKED" => AccountStatus::Blocked,
        "MARKED_FOR_DELETION" | "MARKEDFORDELETION" => AccountStatus::MarkedForDeletion,
        _ => AccountStatus::Active,
    }
}
