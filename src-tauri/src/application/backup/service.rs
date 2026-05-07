use std::sync::Arc;

use crate::application::ports::{
    AuditWriter, Clock, DbSnapshot, EncryptedExporter, EncryptedImporter, SessionStore,
};

#[derive(Clone)]
pub struct BackupService {
    pub(super) snapshot: Arc<dyn DbSnapshot>,
    pub(super) exporter: Arc<dyn EncryptedExporter>,
    pub(super) importer: Arc<dyn EncryptedImporter>,
    pub(super) audit_writer: Arc<dyn AuditWriter>,
    pub(super) clock: Arc<dyn Clock>,
    pub(super) session_store: Arc<dyn SessionStore>,
}

impl BackupService {
    pub fn new(
        snapshot: Arc<dyn DbSnapshot>,
        exporter: Arc<dyn EncryptedExporter>,
        importer: Arc<dyn EncryptedImporter>,
        audit_writer: Arc<dyn AuditWriter>,
        clock: Arc<dyn Clock>,
        session_store: Arc<dyn SessionStore>,
    ) -> Self {
        Self {
            snapshot,
            exporter,
            importer,
            audit_writer,
            clock,
            session_store,
        }
    }
}
