use crate::application::errors::{AppError, AppResult};
use crate::application::session_guard::ensure_admin;
use crate::application::workspace::service::WorkspaceService;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::shared::id::WorkspaceId;

impl WorkspaceService {
    /// admin only。schema 用 ON DELETE CASCADE，删除工作区会一并删除其下配方、
    /// 购物车条目；审计事件保留（FK 设 SET NULL）。
    pub fn delete_workspace(&self, workspace_id: WorkspaceId) -> AppResult<()> {
        let snap = ensure_admin(&*self.session_store)?;
        if let Some(target) = self.workspace_repo.find_by_id(workspace_id)? {
            if target.is_system_mirror() {
                return Err(AppError::Internal("系统内置工作区不可删除".into()));
            }
        }
        self.workspace_repo.delete(workspace_id)?;

        // 如果删的恰好是当前激活 workspace，把会话上的激活也清掉。
        self.session_store.mutate(&mut |s| {
            if s.active_workspace_id() == Some(workspace_id) {
                s.switch_workspace(None);
            }
        });

        // workspace_context_id 在这里必须留空: 工作区已经删了, 而 audit_log
        // 的该列指向 workspaces(id), 写入会触发 FK 约束失败. 工作区 id 留在 target 字段里.
        let event = AuditEvent::new(
            Some(snap.user_id()),
            None,
            Action::WorkspaceDeleted,
            Some(workspace_id.to_string()),
            None,
            self.clock.now(),
        );
        self.audit_writer.record(&event)?;
        Ok(())
    }
}
