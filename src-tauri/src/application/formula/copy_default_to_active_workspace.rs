use crate::application::errors::{AppError, AppResult};
use crate::application::formula::service::FormulaService;
use crate::application::ports::errors::RepositoryError;
use crate::application::session_guard::{ensure_active_workspace, ensure_admin};
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::shared::id::FormulaId;

impl FormulaService {
    pub fn copy_default_to_active_workspace(
        &self,
        default_formula_id: FormulaId,
    ) -> AppResult<FormulaId> {
        let snap = ensure_admin(&*self.session_store)?;
        let (_, workspace_id) = ensure_active_workspace(&*self.session_store)?;
        let default = self
            .default_repo
            .find_by_id(default_formula_id)?
            .ok_or(AppError::Repository(RepositoryError::NotFound))?;

        let new_id = match self.workspace_repo.copy_from_default(&default, workspace_id) {
            Ok(id) => id,
            Err(RepositoryError::Conflict(msg)) => {
                return Err(AppError::Internal(format!(
                    "工作区内已存在同内部色号的配方：{msg}",
                )));
            }
            Err(e) => return Err(AppError::Repository(e)),
        };

        let event = AuditEvent::new(
            Some(snap.user_id()),
            Some(workspace_id),
            Action::DefaultFormulaCopiedToWorkspace,
            Some(default_formula_id.to_string()),
            None,
            self.clock.now(),
        );
        self.audit_writer.record(&event)?;
        Ok(new_id)
    }
}
