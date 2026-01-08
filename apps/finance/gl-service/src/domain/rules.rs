//! Domain Rules for GL Service
//!
//! 业务规则验证逻辑

use crate::domain::value_objects::{DebitCreditIndicator, MonetaryAmount, ValueError};
use rust_decimal::Decimal;

/// 验证凭证行项目借贷平衡
pub fn validate_debit_credit_balance(lines: &[MonetaryAmount]) -> Result<(), ValueError> {
    let mut total_debit = Decimal::ZERO;
    let mut total_credit = Decimal::ZERO;
    
    for line in lines {
        match line.dc_indicator() {
            DebitCreditIndicator::Debit => total_debit += line.amount(),
            DebitCreditIndicator::Credit => total_credit += line.amount(),
        }
    }
    
    if total_debit != total_credit {
        return Err(ValueError::DebitCreditImbalance {
            debit: total_debit,
            credit: total_credit,
        });
    }
    
    Ok(())
}

/// 凭证状态流转规则
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum JournalEntryStatus {
    Draft,
    Parked,
    PendingApproval,
    Approved,
    Posted,
    Reversed,
    Cancelled,
}

impl JournalEntryStatus {
    /// 检查是否可以从当前状态转换到目标状态
    pub fn can_transition_to(&self, target: JournalEntryStatus) -> bool {
        use JournalEntryStatus::*;
        
        matches!(
            (self, target),
            // Draft 可以转换为多种状态
            (Draft, Parked) |
            (Draft, PendingApproval) |
            (Draft, Posted) |
            (Draft, Cancelled) |
            // Parked 可以转换
            (Parked, Draft) |
            (Parked, Posted) |
            (Parked, Cancelled) |
            // PendingApproval 可以转换
            (PendingApproval, Approved) |
            (PendingApproval, Draft) |
            // Approved 可以转换
            (Approved, Posted) |
            (Approved, Draft) |
            // Posted 只能冲销
            (Posted, Reversed)
        )
    }
    
    /// 是否可编辑
    pub fn is_editable(&self) -> bool {
        matches!(self, JournalEntryStatus::Draft | JournalEntryStatus::Parked)
    }
    
    /// 是否可删除
    pub fn is_deletable(&self) -> bool {
        matches!(self, JournalEntryStatus::Draft | JournalEntryStatus::Cancelled)
    }
    
    /// 转换为数据库字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            JournalEntryStatus::Draft => "DRAFT",
            JournalEntryStatus::Parked => "PARKED",
            JournalEntryStatus::PendingApproval => "PENDING_APPROVAL",
            JournalEntryStatus::Approved => "APPROVED",
            JournalEntryStatus::Posted => "POSTED",
            JournalEntryStatus::Reversed => "REVERSED",
            JournalEntryStatus::Cancelled => "CANCELLED",
        }
    }
    
    /// 从数据库字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "DRAFT" => Some(JournalEntryStatus::Draft),
            "PARKED" => Some(JournalEntryStatus::Parked),
            "PENDING_APPROVAL" => Some(JournalEntryStatus::PendingApproval),
            "APPROVED" => Some(JournalEntryStatus::Approved),
            "POSTED" => Some(JournalEntryStatus::Posted),
            "REVERSED" => Some(JournalEntryStatus::Reversed),
            "CANCELLED" => Some(JournalEntryStatus::Cancelled),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_debit_credit_balance_valid() {
        let lines = vec![
            MonetaryAmount::new(Decimal::new(1000, 2), "CNY", DebitCreditIndicator::Debit).unwrap(),
            MonetaryAmount::new(Decimal::new(1000, 2), "CNY", DebitCreditIndicator::Credit).unwrap(),
        ];
        assert!(validate_debit_credit_balance(&lines).is_ok());
    }
    
    #[test]
    fn test_debit_credit_balance_invalid() {
        let lines = vec![
            MonetaryAmount::new(Decimal::new(1000, 2), "CNY", DebitCreditIndicator::Debit).unwrap(),
            MonetaryAmount::new(Decimal::new(500, 2), "CNY", DebitCreditIndicator::Credit).unwrap(),
        ];
        assert!(validate_debit_credit_balance(&lines).is_err());
    }
    
    #[test]
    fn test_status_transitions() {
        assert!(JournalEntryStatus::Draft.can_transition_to(JournalEntryStatus::Posted));
        assert!(JournalEntryStatus::Posted.can_transition_to(JournalEntryStatus::Reversed));
        assert!(!JournalEntryStatus::Posted.can_transition_to(JournalEntryStatus::Draft));
        assert!(!JournalEntryStatus::Reversed.can_transition_to(JournalEntryStatus::Posted));
    }
}
