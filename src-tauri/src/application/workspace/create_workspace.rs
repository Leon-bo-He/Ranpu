use crate::application::errors::{AppError, AppResult};
use crate::application::ports::errors::RepositoryError;
use crate::application::session_guard::ensure_admin;
use crate::application::workspace::service::WorkspaceService;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::shared::id::WorkspaceId;
use crate::domain::workspace::workspace::{Workspace, WorkspaceName};

#[derive(Debug, Clone)]
pub struct CreateWorkspaceInput {
    pub name: String,
    pub description: Option<String>,
}

impl WorkspaceService {
    pub fn create_workspace(&self, input: CreateWorkspaceInput) -> AppResult<WorkspaceId> {
        let snap = ensure_admin(&*self.session_store)?;
        let now = self.clock.now();
        let name = WorkspaceName::new(input.name.clone())?;
        let workspace =
            Workspace::new(name, input.description.clone(), Some(snap.user_id()), now)?;
        let id = match self.workspace_repo.insert(&workspace) {
            Ok(id) => id,
            Err(RepositoryError::Conflict(_)) => {
                return Err(AppError::Internal(format!("工作区名称已存在：{}", input.name)));
            }
            Err(e) => return Err(AppError::Repository(e)),
        };
        let event = AuditEvent::new(
            Some(snap.user_id()),
            Some(id),
            Action::WorkspaceCreated,
            Some(input.name),
            None,
            now,
        );
        self.audit_writer.record(&event)?;
        Ok(id)
    }
}
