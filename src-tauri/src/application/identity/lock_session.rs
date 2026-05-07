use crate::application::errors::{AppError, AppResult};
use crate::application::identity::service::IdentityService;
use crate::domain::audit::audit_event::{Action, AuditEvent};

impl IdentityService {
    /// 手动 / 超时锁屏。
    pub fn lock_session(&self) -> AppResult<()> {
        let mut audit_user = None;
        let locked = self.session_store.mutate(&mut |s| {
            audit_user = Some(s.user_id());
            s.lock();
        });
        if !locked {
            return Err(AppError::NotAuthenticated);
        }
        let event = AuditEvent::new(
            audit_user,
            None,
            Action::SessionLocked,
            None,
            None,
            self.clock.now(),
        );
        self.audit_writer.record(&event)?;
        Ok(())
    }
}
