use crate::application::errors::AppResult;
use crate::application::session_guard::ensure_active;
use crate::application::workspace::service::WorkspaceService;
use crate::domain::workspace::workspace::Workspace;

impl WorkspaceService {
    /// 列出所有工作区. 系统内置 (system_mirror) 工作区始终排在最前,
    /// 其余按 id 升序 (即创建顺序).
    pub fn list_workspaces(&self) -> AppResult<Vec<Workspace>> {
        let _ = ensure_active(&*self.session_store)?;
        let mut all = self.workspace_repo.list_all()?;
        all.sort_by_key(|w| (!w.is_system_mirror(), w.id().map(|i| i.value()).unwrap_or(0)));
        Ok(all)
    }
}
