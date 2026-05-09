use crate::application::errors::{AppError, AppResult};
use crate::application::session_guard::ensure_active;
use crate::application::workspace::service::WorkspaceService;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::shared::id::WorkspaceId;

impl WorkspaceService {
    /// 单用户解锁模型: 解锁后即可切换 workspace.
    /// 传入 None 取消激活；传入 Some 时必须是已存在的 workspace。
    pub fn switch_active_workspace(
        &self,
        workspace_id: Option<WorkspaceId>,
    ) -> AppResult<()> {
        let _ = ensure_active(&*self.session_store)?;
        if let Some(id) = workspace_id {
            if self.workspace_repo.find_by_id(id)?.is_none() {
                return Err(AppError::Internal(format!(
                    "找不到工作区 (id={})",
                    id.value()
                )));
            }
        }
        self.session_store
            .mutate(&mut |s| s.switch_workspace(workspace_id));
        let event = AuditEvent::new(
            workspace_id,
            Action::WorkspaceSwitched,
            workspace_id.map(|id| id.to_string()),
            None,
            self.clock.now(),
        );
        self.audit_writer.record(&event)?;
        Ok(())
    }
}
