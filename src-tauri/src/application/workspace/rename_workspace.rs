use crate::application::errors::{AppError, AppResult};
use crate::application::session_guard::ensure_active;
use crate::application::workspace::service::WorkspaceService;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::shared::id::WorkspaceId;
use crate::domain::workspace::workspace::WorkspaceName;

#[derive(Debug, Clone)]
pub struct RenameWorkspaceInput {
    pub workspace_id: WorkspaceId,
    pub new_name: String,
}

impl WorkspaceService {
    pub fn rename_workspace(&self, input: RenameWorkspaceInput) -> AppResult<()> {
        let _ = ensure_active(&*self.session_store)?;
        let name = WorkspaceName::new(input.new_name.clone())?;
        if let Some(target) = self.workspace_repo.find_by_id(input.workspace_id)? {
            if target.is_system_mirror() {
                return Err(AppError::Internal(
                    "系统内置工作区不可重命名".into(),
                ));
            }
        }
        self.workspace_repo
            .rename(input.workspace_id, name.as_str())?;
        let event = AuditEvent::new(
            Some(input.workspace_id),
            Action::WorkspaceRenamed,
            Some(input.new_name),
            None,
            self.clock.now(),
        );
        self.audit_writer.record(&event)?;
        Ok(())
    }
}
