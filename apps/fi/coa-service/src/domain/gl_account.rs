// 会计科目领域模型
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 科目性质
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountNature {
    Asset,        // 资产
    Liability,    // 负债
    Equity,       // 权益
    Revenue,      // 收入
    Expense,      // 费用
    ProfitLoss,   // 损益
}

/// 科目状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountStatus {
    Active,              // 激活
    Inactive,            // 停用
    Blocked,             // 冻结
    MarkedForDeletion,   // 标记为删除
}

/// 余额方向
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BalanceIndicator {
    Debit,   // 借方
    Credit,  // 贷方
}

/// 会计科目聚合根
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlAccount {
    pub id: Uuid,
    pub chart_code: String,
    pub account_code: String,

    // 基础信息
    pub account_name: String,
    pub account_name_long: Option<String>,
    pub search_key: Option<String>,

    // 分类
    pub account_nature: AccountNature,
    pub account_category: String,
    pub account_group: Option<String>,

    // 层级
    pub account_level: i32,
    pub parent_account: Option<String>,
    pub is_leaf_account: bool,
    pub full_path: Option<String>,
    pub depth: i32,

    // 控制
    pub is_postable: bool,
    pub is_cost_element: bool,
    pub line_item_display: bool,
    pub open_item_management: bool,
    pub balance_indicator: BalanceIndicator,

    // 财务属性
    pub currency: Option<String>,
    pub only_local_currency: bool,
    pub exchange_rate_diff: bool,

    // 税务
    pub tax_relevant: bool,
    pub tax_category: Option<String>,

    // 状态
    pub status: AccountStatus,
    pub valid_from: Option<NaiveDate>,
    pub valid_to: Option<NaiveDate>,

    // 审计
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub changed_by: Option<String>,
    pub changed_at: Option<DateTime<Utc>>,
    pub tenant_id: Option<String>,
}

impl GlAccount {
    /// 创建新科目
    pub fn new(
        chart_code: String,
        account_code: String,
        account_name: String,
        account_nature: AccountNature,
        account_category: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            chart_code,
            account_code,
            account_name,
            account_name_long: None,
            search_key: None,
            account_nature,
            account_category,
            account_group: None,
            account_level: 1,
            parent_account: None,
            is_leaf_account: true,
            full_path: None,
            depth: 0,
            is_postable: true,
            is_cost_element: false,
            line_item_display: true,
            open_item_management: false,
            balance_indicator: BalanceIndicator::Debit,
            currency: None,
            only_local_currency: true,
            exchange_rate_diff: false,
            tax_relevant: false,
            tax_category: None,
            status: AccountStatus::Active,
            valid_from: None,
            valid_to: None,
            created_by: None,
            created_at: Utc::now(),
            changed_by: None,
            changed_at: None,
            tenant_id: None,
        }
    }

    /// 验证科目代码格式（10位数字）
    pub fn validate_account_code(&self) -> Result<(), String> {
        if self.account_code.len() > 10 {
            return Err("科目代码长度不能超过10位".to_string());
        }
        if !self.account_code.chars().all(|c| c.is_ascii_digit()) {
            return Err("科目代码只能包含数字".to_string());
        }
        Ok(())
    }

    /// 检查科目是否可过账
    pub fn is_postable_at(&self, posting_date: NaiveDate) -> bool {
        // 检查状态
        if self.status != AccountStatus::Active {
            return false;
        }

        // 检查是否可过账标识
        if !self.is_postable {
            return false;
        }

        // 检查有效期
        if let Some(valid_from) = self.valid_from {
            if posting_date < valid_from {
                return false;
            }
        }

        if let Some(valid_to) = self.valid_to {
            if posting_date > valid_to {
                return false;
            }
        }

        true
    }

    /// 激活科目
    pub fn activate(&mut self) {
        self.status = AccountStatus::Active;
        self.changed_at = Some(Utc::now());
    }

    /// 停用科目
    pub fn deactivate(&mut self) {
        self.status = AccountStatus::Inactive;
        self.changed_at = Some(Utc::now());
    }

    /// 冻结科目
    pub fn block(&mut self) {
        self.status = AccountStatus::Blocked;
        self.changed_at = Some(Utc::now());
    }
}

/// 公司代码级科目数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyCodeAccountData {
    pub id: Uuid,
    pub company_code: String,
    pub account_code: String,
    pub chart_code: String,

    // 公司代码级控制
    pub posting_blocked: bool,
    pub reconciliation_account_type: Option<String>,
    pub field_status_group: Option<String>,
    pub automatic_postings: bool,
    pub sort_key: Option<String>,
    pub tax_code: Option<String>,
    pub cash_flow_category: Option<String>,

    // 审计
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub changed_by: Option<String>,
    pub changed_at: Option<DateTime<Utc>>,
}

/// 科目文本（多语言）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountText {
    pub id: Uuid,
    pub chart_code: String,
    pub account_code: String,
    pub language_code: String,
    pub short_text: Option<String>,
    pub medium_text: Option<String>,
    pub long_text: Option<String>,
    pub description: Option<String>,
}
