use crate::application::errors::{AppError, AppResult};
use crate::application::identity::service::IdentityService;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::shared::id::UserId;

impl IdentityService {
    /// admin only。不允许停用自己（避免锁死系统）。
    pub fn deactivate_user(&self, target_user_id: UserId) -> AppResult<()> {
        let snap = self
            .session_store
            .current()
            .ok_or(AppError::NotAuthenticated)?;
        if snap.is_locked() {
            return Err(AppError::SessionLocked);
        }
        if !snap.role().can_manage_users() {
            return Err(AppError::PermissionDenied);
        }
        if snap.user_id() == target_user_id {
            return Err(AppError::Internal("不能停用当前登录的自己".to_owned()));
        }
        self.user_repo.set_active(target_user_id, false)?;
        let event = AuditEvent::new(
            Some(snap.user_id()),
            None,
            Action::UserDeactivated,
            Some(target_user_id.to_string()),
            None,
            self.clock.now(),
        );
        self.audit_writer.record(&event)?;
        Ok(())
    }
}
