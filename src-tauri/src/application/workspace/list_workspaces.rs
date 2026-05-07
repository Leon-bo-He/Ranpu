use crate::application::errors::AppResult;
use crate::application::session_guard::ensure_active;
use crate::application::workspace::service::WorkspaceService;
use crate::domain::workspace::workspace::Workspace;

impl WorkspaceService {
    pub fn list_workspaces(&self) -> AppResult<Vec<Workspace>> {
        let _ = ensure_active(&*self.session_store)?;
        Ok(self.workspace_repo.list_all()?)
    }
}
