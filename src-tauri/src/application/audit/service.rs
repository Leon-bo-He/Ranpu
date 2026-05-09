use std::sync::Arc;

use crate::application::errors::AppResult;
use crate::application::ports::{
    AuditCsvExporter, AuditRepository, AuditWriter, Clock, EncryptedExporter, SessionStore,
};
use crate::domain::audit::audit_event::AuditEvent;

#[derive(Clone)]
pub struct AuditService {
    pub(super) audit_repo: Arc<dyn AuditRepository>,
    pub(super) audit_writer: Arc<dyn AuditWriter>,
    pub(super) csv_exporter: Arc<dyn AuditCsvExporter>,
    pub(super) encrypted_exporter: Arc<dyn EncryptedExporter>,
    pub(super) clock: Arc<dyn Clock>,
    pub(super) session_store: Arc<dyn SessionStore>,
}

impl AuditService {
    /// 直接写一条审计事件 — interfaces 层的 cmd_lock_session / cmd_unlock_session
    /// 需要在没经过具体 use case 的情况下补一笔事件.
    pub fn write_event(&self, event: &AuditEvent) -> AppResult<()> {
        Ok(self.audit_writer.record(event)?)
    }
}

impl AuditService {
    pub fn new(
        audit_repo: Arc<dyn AuditRepository>,
        audit_writer: Arc<dyn AuditWriter>,
        csv_exporter: Arc<dyn AuditCsvExporter>,
        encrypted_exporter: Arc<dyn EncryptedExporter>,
        clock: Arc<dyn Clock>,
        session_store: Arc<dyn SessionStore>,
    ) -> Self {
        Self {
            audit_repo,
            audit_writer,
            csv_exporter,
            encrypted_exporter,
            clock,
            session_store,
        }
    }
}
