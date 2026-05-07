use crate::application::errors::AppResult;
use crate::application::identity::service::IdentityService;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::shared::id::UserId;

impl IdentityService {
    /// admin only。把已停用的用户重新启用，恢复登录权限。
    pub fn activate_user(&self, target_user_id: UserId) -> AppResult<()> {
        let snap = crate::application::session_guard::ensure_admin(&*self.session_store)?;
        self.user_repo.set_active(target_user_id, true)?;
        let event = AuditEvent::new(
            Some(snap.user_id()),
            None,
            Action::UserActivated,
            Some(target_user_id.to_string()),
            None,
            self.clock.now(),
        );
        self.audit_writer.record(&event)?;
        Ok(())
    }
}
