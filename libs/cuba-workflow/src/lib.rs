//! cuba-workflow - 通用工作流和状态历史追踪库
//!
//! 提供单据状态变更的审计追踪能力，支持：
//! - AP 发票审批/拒绝
//! - AR 发票审批/拒绝
//! - PO 采购订单审批
//! - SO 销售订单审批
//! - 其他需要状态追踪的业务单据

mod repository;
mod status_history;

pub use repository::{
    PgStatusHistoryRepository, RepositoryError, StatusHistoryRepository, create_table_sql,
};
pub use status_history::{ActionType, StandardAction, StatusHistory, StatusHistoryBuilder};
