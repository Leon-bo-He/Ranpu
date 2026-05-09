use crate::application::errors::{AppError, AppResult};
use crate::application::session_guard::ensure_active;
use crate::application::workspace::service::WorkspaceService;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::shared::id::WorkspaceId;

#[derive(Debug, Clone)]
pub struct UpdateWorkspaceDescriptionInput {
    pub workspace_id: WorkspaceId,
    /// None / 空白 → 清空说明.
    pub description: Option<String>,
}

impl WorkspaceService {
    pub fn update_workspace_description(
        &self,
        input: UpdateWorkspaceDescriptionInput,
    ) -> AppResult<()> {
        let _ = ensure_active(&*self.session_store)?;
        if let Some(target) = self.workspace_repo.find_by_id(input.workspace_id)? {
            if target.is_system_mirror() {
                return Err(AppError::Internal(
                    "系统内置工作区说明不可修改".into(),
                ));
            }
        }
        // 与 Workspace::set_description 一致: trim, 空 → None, 限长 1024.
        let normalized = match input.description.as_deref() {
            None => None,
            Some(s) => {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    if trimmed.chars().count() > 1024 {
                        return Err(AppError::Internal("说明过长 (最多 1024 字)".into()));
                    }
                    Some(trimmed.to_owned())
                }
            }
        };
        self.workspace_repo
            .update_description(input.workspace_id, normalized.as_deref())?;
        let event = AuditEvent::new(
            Some(input.workspace_id),
            Action::WorkspaceDescriptionUpdated,
            Some(input.workspace_id.to_string()),
            None,
            self.clock.now(),
        );
        self.audit_writer.record(&event)?;
        Ok(())
    }
}
