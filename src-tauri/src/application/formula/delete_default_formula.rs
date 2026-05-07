use crate::application::errors::AppResult;
use crate::application::formula::service::FormulaService;
use crate::application::session_guard::ensure_admin;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::shared::id::FormulaId;

impl FormulaService {
    pub fn delete_default_formula(&self, id: FormulaId) -> AppResult<()> {
        let snap = ensure_admin(&*self.session_store)?;
        self.default_repo.delete(id)?;
        let event = AuditEvent::new(
            Some(snap.user_id()),
            None,
            Action::DefaultFormulaDeleted,
            Some(id.to_string()),
            None,
            self.clock.now(),
        );
        self.audit_writer.record(&event)?;
        Ok(())
    }
}
