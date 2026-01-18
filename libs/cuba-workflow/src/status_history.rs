//! 状态历史实体和操作类型定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 通用操作类型 trait
/// 各模块可实现自己的操作类型枚举
pub trait ActionType: Send + Sync {
    /// 转换为字符串存储
    fn as_str(&self) -> &str;

    /// 从字符串解析
    fn from_str(s: &str) -> Option<Self>
    where
        Self: Sized;

    /// 获取操作描述
    fn description(&self) -> &str;
}

/// 预定义的通用操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StandardAction {
    Create,   // 创建
    Submit,   // 提交
    Approve,  // 审批通过
    Reject,   // 拒绝
    Clear,    // 清账
    Reverse,  // 冲销
    Reopen,   // 重新打开
    Cancel,   // 取消
    Complete, // 完成
    Close,    // 关闭
}

impl ActionType for StandardAction {
    fn as_str(&self) -> &str {
        match self {
            Self::Create => "CREATE",
            Self::Submit => "SUBMIT",
            Self::Approve => "APPROVE",
            Self::Reject => "REJECT",
            Self::Clear => "CLEAR",
            Self::Reverse => "REVERSE",
            Self::Reopen => "REOPEN",
            Self::Cancel => "CANCEL",
            Self::Complete => "COMPLETE",
            Self::Close => "CLOSE",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "CREATE" => Some(Self::Create),
            "SUBMIT" => Some(Self::Submit),
            "APPROVE" => Some(Self::Approve),
            "REJECT" => Some(Self::Reject),
            "CLEAR" => Some(Self::Clear),
            "REVERSE" => Some(Self::Reverse),
            "REOPEN" => Some(Self::Reopen),
            "CANCEL" => Some(Self::Cancel),
            "COMPLETE" => Some(Self::Complete),
            "CLOSE" => Some(Self::Close),
            _ => None,
        }
    }

    fn description(&self) -> &str {
        match self {
            Self::Create => "创建",
            Self::Submit => "提交",
            Self::Approve => "审批通过",
            Self::Reject => "拒绝",
            Self::Clear => "清账",
            Self::Reverse => "冲销",
            Self::Reopen => "重新打开",
            Self::Cancel => "取消",
            Self::Complete => "完成",
            Self::Close => "关闭",
        }
    }
}

/// 通用状态历史记录
/// 用于记录任何业务单据的状态变更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusHistory {
    /// 记录ID
    pub id: Uuid,
    /// 关联的单据ID（发票、订单等）
    pub document_id: Uuid,
    /// 单据类型（INVOICE, PO, SO 等）
    pub document_type: String,
    /// 原状态（首次创建为 None）
    pub from_status: Option<String>,
    /// 新状态
    pub to_status: String,
    /// 变更原因（拒绝原因等）
    pub reason: Option<String>,
    /// 操作类型
    pub action_type: String,
    /// 操作人ID
    pub changed_by: Option<String>,
    /// 操作人名称
    pub changed_by_name: Option<String>,
    /// 变更时间
    pub changed_at: DateTime<Utc>,
    /// 备注
    pub remarks: Option<String>,
    /// 扩展元数据（JSON）
    pub metadata: Option<String>,
}

// 为 sqlx::FromRow 实现
impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for StatusHistory {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(Self {
            id: row.try_get("id")?,
            document_id: row.try_get("document_id")?,
            document_type: row.try_get("document_type")?,
            from_status: row.try_get("from_status")?,
            to_status: row.try_get("to_status")?,
            reason: row.try_get("reason")?,
            action_type: row.try_get("action_type")?,
            changed_by: row.try_get("changed_by")?,
            changed_by_name: row.try_get("changed_by_name")?,
            changed_at: row.try_get("changed_at")?,
            remarks: row.try_get("remarks")?,
            metadata: row.try_get("metadata")?,
        })
    }
}

impl StatusHistory {
    /// 使用 Builder 创建状态历史
    pub fn builder(document_id: Uuid, document_type: impl Into<String>) -> StatusHistoryBuilder {
        StatusHistoryBuilder::new(document_id, document_type)
    }
}

/// 状态历史构建器
pub struct StatusHistoryBuilder {
    document_id: Uuid,
    document_type: String,
    from_status: Option<String>,
    to_status: Option<String>,
    reason: Option<String>,
    action_type: Option<String>,
    changed_by: Option<String>,
    changed_by_name: Option<String>,
    remarks: Option<String>,
    metadata: Option<String>,
}

impl StatusHistoryBuilder {
    pub fn new(document_id: Uuid, document_type: impl Into<String>) -> Self {
        Self {
            document_id,
            document_type: document_type.into(),
            from_status: None,
            to_status: None,
            reason: None,
            action_type: None,
            changed_by: None,
            changed_by_name: None,
            remarks: None,
            metadata: None,
        }
    }

    /// 设置原状态
    pub fn from_status(mut self, status: impl Into<String>) -> Self {
        self.from_status = Some(status.into());
        self
    }

    /// 设置新状态
    pub fn to_status(mut self, status: impl Into<String>) -> Self {
        self.to_status = Some(status.into());
        self
    }

    /// 设置操作类型（使用 ActionType trait）
    pub fn action<A: ActionType>(mut self, action: A) -> Self {
        self.action_type = Some(action.as_str().to_string());
        self
    }

    /// 设置操作类型（使用字符串）
    pub fn action_str(mut self, action: impl Into<String>) -> Self {
        self.action_type = Some(action.into());
        self
    }

    /// 设置变更原因
    pub fn reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// 设置操作人
    pub fn changed_by(mut self, user_id: impl Into<String>) -> Self {
        self.changed_by = Some(user_id.into());
        self
    }

    /// 设置操作人名称
    pub fn changed_by_name(mut self, name: impl Into<String>) -> Self {
        self.changed_by_name = Some(name.into());
        self
    }

    /// 设置备注
    pub fn remarks(mut self, remarks: impl Into<String>) -> Self {
        self.remarks = Some(remarks.into());
        self
    }

    /// 设置扩展元数据
    pub fn metadata(mut self, metadata: impl Into<String>) -> Self {
        self.metadata = Some(metadata.into());
        self
    }

    /// 构建状态历史记录
    pub fn build(self) -> Result<StatusHistory, &'static str> {
        let to_status = self.to_status.ok_or("to_status is required")?;
        let action_type = self.action_type.ok_or("action_type is required")?;

        Ok(StatusHistory {
            id: Uuid::new_v4(),
            document_id: self.document_id,
            document_type: self.document_type,
            from_status: self.from_status,
            to_status,
            reason: self.reason,
            action_type,
            changed_by: self.changed_by,
            changed_by_name: self.changed_by_name,
            changed_at: Utc::now(),
            remarks: self.remarks,
            metadata: self.metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_status_history() {
        let history = StatusHistory::builder(Uuid::new_v4(), "INVOICE")
            .from_status("OPEN")
            .to_status("APPROVED")
            .action(StandardAction::Approve)
            .changed_by("user123")
            .build()
            .unwrap();

        assert_eq!(history.document_type, "INVOICE");
        assert_eq!(history.from_status, Some("OPEN".to_string()));
        assert_eq!(history.to_status, "APPROVED");
        assert_eq!(history.action_type, "APPROVE");
    }

    #[test]
    fn test_reject_with_reason() {
        let history = StatusHistory::builder(Uuid::new_v4(), "INVOICE")
            .from_status("PENDING")
            .to_status("REJECTED")
            .action(StandardAction::Reject)
            .reason("金额不符")
            .changed_by("approver001")
            .build()
            .unwrap();

        assert_eq!(history.to_status, "REJECTED");
        assert_eq!(history.reason, Some("金额不符".to_string()));
    }
}
