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
    async fn create(&self, account: &GlAccount) -> anyhow::Result<()>;
    async fn update(&self, account: &GlAccount) -> anyhow::Result<()>;
    async fn delete(&self, chart_code: &str, account_code: &str) -> anyhow::Result<()>;
    async fn find_by_code(&self, chart_code: &str, account_code: &str) -> anyhow::Result<Option<GlAccount>>;
    async fn find_all(&self, chart_code: &str) -> anyhow::Result<Vec<GlAccount>>;
    async fn find_by_nature(&self, chart_code: &str, nature: &AccountNature) -> anyhow::Result<Vec<GlAccount>>;
    async fn find_postable_accounts(&self, chart_code: &str) -> anyhow::Result<Vec<GlAccount>>;
    async fn validate_account(&self, chart_code: &str, account_code: &str, posting_date: NaiveDate) -> anyhow::Result<AccountValidationResult>;
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
    async fn create(&self, account: &GlAccount) -> anyhow::Result<()> {
        sqlx::query(
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
        )
        .bind(account.id)
        .bind(&account.chart_code)
        .bind(&account.account_code)
        .bind(&account.account_name)
        .bind(&account.account_name_long)
        .bind(account.account_nature.clone())
        .bind(&account.account_category)
        .bind(&account.account_group)
        .bind(account.account_level)
        .bind(&account.parent_account)
        .bind(account.is_leaf_account)
        .bind(&account.full_path)
        .bind(account.depth)
        .bind(account.is_postable)
        .bind(account.is_cost_element)
        .bind(account.line_item_display)
        .bind(account.open_item_management)
        .bind(account.balance_indicator.clone())
        .bind(&account.currency)
        .bind(account.only_local_currency)
        .bind(account.exchange_rate_diff)
        .bind(account.tax_relevant)
        .bind(&account.tax_category)
        .bind(account.status.clone())
        .bind(account.valid_from)
        .bind(account.valid_to)
        .bind(&account.created_by)
        .bind(account.created_at)
        .bind(&account.tenant_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update(&self, account: &GlAccount) -> anyhow::Result<()> {
        sqlx::query(
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
        )
        .bind(&account.account_name)
        .bind(&account.account_name_long)
        .bind(account.account_nature.clone())
        .bind(&account.account_category)
        .bind(account.status.clone())
        .bind(&account.changed_by)
        .bind(account.changed_at)
        .bind(&account.chart_code)
        .bind(&account.account_code)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, chart_code: &str, account_code: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM gl_accounts WHERE chart_code = $1 AND account_code = $2")
            .bind(chart_code)
            .bind(account_code)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn find_by_code(&self, chart_code: &str, account_code: &str) -> anyhow::Result<Option<GlAccount>> {
        let row = sqlx::query_as::<_, GlAccount>(
            r#"
            SELECT * FROM gl_accounts
            WHERE chart_code = $1 AND account_code = $2
            "#,
        )
        .bind(chart_code)
        .bind(account_code)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_all(&self, chart_code: &str) -> anyhow::Result<Vec<GlAccount>> {
        let rows = sqlx::query_as::<_, GlAccount>(
            "SELECT * FROM gl_accounts WHERE chart_code = $1 ORDER BY account_code",
        )
        .bind(chart_code)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn find_by_nature(&self, chart_code: &str, nature: &AccountNature) -> anyhow::Result<Vec<GlAccount>> {
        let nature_str = format!("{:?}", nature).to_uppercase();
        let rows = sqlx::query_as::<_, GlAccount>(
            "SELECT * FROM gl_accounts WHERE chart_code = $1 AND account_nature = $2",
        )
        .bind(chart_code)
        .bind(nature_str)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn find_postable_accounts(&self, chart_code: &str) -> anyhow::Result<Vec<GlAccount>> {
        let rows = sqlx::query(
            "SELECT * FROM vw_postable_accounts WHERE chart_code = $1",
        )
        .bind(chart_code)
        .fetch_all(&self.pool)
        .await?;

        use sqlx::Row;
        Ok(rows.into_iter().map(|r| GlAccount {
            id: Uuid::new_v4(), // 视图没有ID，临时生成
            chart_code: r.get::<Option<String>, _>("chart_code").unwrap_or_default(),
            account_code: r.get::<Option<String>, _>("account_code").unwrap_or_default(),
            account_name: r.get::<Option<String>, _>("account_name").unwrap_or_default(),
            account_name_long: None,
            search_key: None,
            account_nature: r.get::<Option<String>, _>("account_nature")
                .map(|s| parse_account_nature(&s))
                .unwrap_or(AccountNature::Asset),
            account_category: String::new(),
            account_group: None,
            account_level: r.get::<Option<i32>, _>("account_level").unwrap_or(1),
            parent_account: r.get::<Option<String>, _>("parent_account"),
            is_leaf_account: true,
            full_path: None,
            depth: 0,
            is_postable: true,
            is_cost_element: false,
            line_item_display: true,
            open_item_management: false,
            balance_indicator: r.get::<Option<String>, _>("balance_indicator")
                .map(|s| if s == "C" {
                    BalanceIndicator::Credit
                } else {
                    BalanceIndicator::Debit
                }).unwrap_or(BalanceIndicator::Debit),
            currency: None,
            only_local_currency: true,
            exchange_rate_diff: false,
            tax_relevant: false,
            tax_category: None,
            status: r.get::<Option<String>, _>("status")
                .map(|s| parse_account_status(&s))
                .unwrap_or(AccountStatus::Active),
            valid_from: None,
            valid_to: None,
            created_by: None,
            created_at: chrono::Utc::now(),
            changed_by: None,
            changed_at: None,
            tenant_id: None,
        }).collect())
    }

    async fn validate_account(&self, chart_code: &str, account_code: &str, posting_date: NaiveDate) -> anyhow::Result<AccountValidationResult> {
        let result = sqlx::query("SELECT * FROM fn_validate_account($1, $2, $3)")
            .bind(chart_code)
            .bind(account_code)
            .bind(posting_date)
            .fetch_one(&self.pool)
            .await?;

        use sqlx::Row;
        Ok(AccountValidationResult {
            is_valid: result.get::<Option<bool>, _>("is_valid").unwrap_or(false),
            exists: result.get::<Option<bool>, _>("exists").unwrap_or(false),
            is_active: result.get::<Option<bool>, _>("is_active").unwrap_or(false),
            is_postable: result.get::<Option<bool>, _>("is_postable").unwrap_or(false),
            error_message: result.get::<Option<String>, _>("error_message"),
        })
    }
}

// Helper functions (Internal for mapping from raw strings if needed)
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
