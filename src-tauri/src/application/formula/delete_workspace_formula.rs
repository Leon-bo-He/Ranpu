use crate::application::errors::AppResult;
use crate::application::formula::service::FormulaService;
use crate::application::session_guard::{ensure_active_workspace, ensure_admin};
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::shared::id::FormulaId;

impl FormulaService {
    pub fn delete_workspace_formula(&self, id: FormulaId) -> AppResult<()> {
        let snap = ensure_admin(&*self.session_store)?;
        let (_, workspace_id) = ensure_active_workspace(&*self.session_store)?;
        self.workspace_repo.delete(workspace_id, id)?;
        let event = AuditEvent::new(
            Some(snap.user_id()),
            Some(workspace_id),
            Action::WorkspaceFormulaDeleted,
            Some(id.to_string()),
            None,
            self.clock.now(),
        );
        self.audit_writer.record(&event)?;
        Ok(())
    }
}
