//! Get Audit Logs Handler

use crate::domain::repositories::{AuditLogRepository, AuditLogData, RepositoryError};
use std::sync::Arc;

pub struct GetAuditLogsCommand {
    pub user_id: Option<String>,
    pub tenant_id: Option<String>,
    pub action: Option<String>,
    pub resource: Option<String>,
    pub context_user_id: Option<String>,
    pub context_tenant_id: Option<String>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: i64,
    pub offset: i64,
}

pub struct GetAuditLogsResponse {
    pub logs: Vec<AuditLogData>,
    pub total: i64,
}

pub struct GetAuditLogsHandler {
    audit_repo: Arc<dyn AuditLogRepository>,
}

impl GetAuditLogsHandler {
    pub fn new(audit_repo: Arc<dyn AuditLogRepository>) -> Self {
        Self { audit_repo }
    }

    pub async fn handle(&self, command: GetAuditLogsCommand) -> Result<GetAuditLogsResponse, RepositoryError> {
        let logs = self.audit_repo.find_logs(
            command.user_id.as_deref(),
            command.tenant_id.as_deref(),
            command.action.as_deref(),
            command.start_time,
            command.end_time,
            command.limit,
            command.offset,
        ).await?;

        let total = self.audit_repo.count_logs(
            command.user_id.as_deref(),
            command.tenant_id.as_deref(),
            command.action.as_deref(),
            command.start_time,
            command.end_time,
        ).await?;

        Ok(GetAuditLogsResponse { logs, total })
    }
}
