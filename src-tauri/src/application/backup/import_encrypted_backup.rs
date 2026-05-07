use std::path::PathBuf;

use crate::application::backup::service::BackupService;
use crate::application::errors::{AppError, AppResult};
use crate::application::session_guard::ensure_admin;
use crate::domain::audit::audit_event::{Action, AuditEvent};

#[derive(Debug, Clone)]
pub struct ImportBackupInput {
    pub passphrase: String,
    pub in_path: PathBuf,
}

impl BackupService {
    pub fn import_encrypted_backup(&self, input: ImportBackupInput) -> AppResult<()> {
        let snap = ensure_admin(&*self.session_store)?;
        let bytes = self
            .importer
            .import_from_file(&input.in_path, &input.passphrase)
            .map_err(|e| AppError::Crypto(e.to_string()))?;
        self.snapshot
            .restore_bytes(&bytes)
            .map_err(|e| AppError::Io(e.to_string()))?;
        let event = AuditEvent::new(
            Some(snap.user_id()),
            None,
            Action::BackupImported,
            Some(input.in_path.to_string_lossy().into_owned()),
            None,
            self.clock.now(),
        );
        self.audit_writer.record(&event)?;
        Ok(())
    }
}
