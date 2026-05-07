use std::sync::Arc;

use crate::application::ports::{AuditWriter, Clock, SessionStore, WorkspaceRepository};

#[derive(Clone)]
pub struct WorkspaceService {
    pub(super) workspace_repo: Arc<dyn WorkspaceRepository>,
    pub(super) audit_writer: Arc<dyn AuditWriter>,
    pub(super) clock: Arc<dyn Clock>,
    pub(super) session_store: Arc<dyn SessionStore>,
}

impl WorkspaceService {
    pub fn new(
        workspace_repo: Arc<dyn WorkspaceRepository>,
        audit_writer: Arc<dyn AuditWriter>,
        clock: Arc<dyn Clock>,
        session_store: Arc<dyn SessionStore>,
    ) -> Self {
        Self {
            workspace_repo,
            audit_writer,
            clock,
            session_store,
        }
    }
}
