use crate::application::ports::errors::RepositoryError;
use crate::domain::audit::audit_event::AuditEvent;

/// 审计事件写入抽象（与 AuditRepository 分开，明确意图）。
pub trait AuditWriter: Send + Sync {
    fn record(&self, event: &AuditEvent) -> Result<(), RepositoryError>;
}
