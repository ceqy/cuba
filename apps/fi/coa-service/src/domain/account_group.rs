// 科目组领域模型
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AccountNature;

/// 科目组
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountGroup {
    pub id: Uuid,
    pub chart_code: String,
    pub group_code: String,

    // 基础信息
    pub group_name: String,
    pub description: Option<String>,

    // 科目性质
    pub account_nature: Option<AccountNature>,

    // 编号范围
    pub number_range_from: Option<String>,
    pub number_range_to: Option<String>,

    // 字段状态控制
    pub field_status_variant: Option<String>,

    // 审计
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub changed_by: Option<String>,
    pub changed_at: Option<DateTime<Utc>>,
    pub tenant_id: Option<String>,
}

impl AccountGroup {
    /// 创建新科目组
    pub fn new(chart_code: String, group_code: String, group_name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            chart_code,
            group_code,
            group_name,
            description: None,
            account_nature: None,
            number_range_from: None,
            number_range_to: None,
            field_status_variant: None,
            created_by: None,
            created_at: Utc::now(),
            changed_by: None,
            changed_at: None,
            tenant_id: None,
        }
    }

    /// 检查科目代码是否在编号范围内
    pub fn is_account_in_range(&self, account_code: &str) -> bool {
        let from_ok = match &self.number_range_from {
            Some(from) => account_code >= from.as_str(),
            None => true,
        };

        let to_ok = match &self.number_range_to {
            Some(to) => account_code <= to.as_str(),
            None => true,
        };

        from_ok && to_ok
    }
}
