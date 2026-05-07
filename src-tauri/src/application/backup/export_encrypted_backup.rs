use std::path::PathBuf;

use crate::application::backup::service::BackupService;
use crate::application::errors::{AppError, AppResult};
use crate::application::session_guard::ensure_admin;
use crate::domain::audit::audit_event::{Action, AuditEvent};

#[derive(Debug, Clone)]
pub struct ExportBackupInput {
    pub passphrase: String,
    pub out_path: PathBuf,
}

impl BackupService {
    pub fn export_encrypted_backup(&self, input: ExportBackupInput) -> AppResult<()> {
        let snap = ensure_admin(&*self.session_store)?;
        let bytes = self
            .snapshot
            .snapshot_bytes()
            .map_err(|e| AppError::Io(e.to_string()))?;
        self.exporter
            .export_to_file(&bytes, &input.passphrase, &input.out_path)
            .map_err(|e| AppError::Crypto(e.to_string()))?;
        let event = AuditEvent::new(
            Some(snap.user_id()),
            None,
            Action::BackupExported,
            Some(input.out_path.to_string_lossy().into_owned()),
            None,
            self.clock.now(),
        );
        self.audit_writer.record(&event)?;
        Ok(())
    }
}
