use crate::application::errors::AppResult;
use crate::application::formula::service::FormulaService;
use crate::application::session_guard::ensure_active_workspace;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::shared::id::FormulaId;

impl FormulaService {
    pub fn delete_workspace_formula(&self, id: FormulaId) -> AppResult<()> {
        let (_, workspace_id) = ensure_active_workspace(&*self.session_store)?;
        self.reject_if_system_mirror(workspace_id)?;
        self.workspace_repo.delete(workspace_id, id)?;
        let event = AuditEvent::new(
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
