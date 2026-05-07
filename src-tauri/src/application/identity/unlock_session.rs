use crate::application::errors::{AppError, AppResult};
use crate::application::identity::service::IdentityService;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::identity::errors::IdentityError;
use crate::domain::identity::session::UNLOCK_FAILURE_LIMIT;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnlockOutcome {
    Unlocked,
    /// 解锁失败但还有机会，剩余 N 次。
    StillLocked { remaining: u32 },
    /// 解锁失败累计达到上限，已强制登出。
    ForceLoggedOut,
}

#[derive(Debug, Clone)]
pub struct UnlockSessionInput {
    pub password: String,
}

impl IdentityService {
    pub fn unlock_session(&self, input: UnlockSessionInput) -> AppResult<UnlockOutcome> {
        let now = self.clock.now();
        let snap = self
            .session_store
            .current()
            .ok_or(AppError::NotAuthenticated)?;
        if !snap.is_locked() {
            return Err(AppError::Identity(IdentityError::SessionLocked));
        }

        // 校验密码。
        let user = self
            .user_repo
            .find_by_id(snap.user_id())?
            .ok_or(AppError::Identity(IdentityError::NotAuthenticated))?;
        let password_ok = self
            .hasher
            .verify(&input.password, user.password_hash())
            .map_err(|e| AppError::Crypto(e.to_string()))?;

        if password_ok {
            // 解锁。
            self.session_store.mutate(&mut |s| s.unlock(now));
            let event = AuditEvent::new(
                Some(snap.user_id()),
                snap.active_workspace_id(),
                Action::SessionUnlocked,
                None,
                None,
                now,
            );
            self.audit_writer.record(&event)?;
            return Ok(UnlockOutcome::Unlocked);
        }

        // 错误：累计失败 + 看是否达到上限。
        let mut remaining = 0;
        let mut force_out = false;
        self.session_store.mutate(&mut |s| {
            remaining = s.record_unlock_failure();
            if s.should_force_logout() {
                force_out = true;
            }
        });

        if force_out {
            self.session_store.clear();
            let event = AuditEvent::new(
                Some(snap.user_id()),
                snap.active_workspace_id(),
                Action::SessionForceLogout,
                None,
                None,
                now,
            );
            self.audit_writer.record(&event)?;
            return Ok(UnlockOutcome::ForceLoggedOut);
        }

        // 记审计也写一笔失败。
        let event = AuditEvent::new(
            Some(snap.user_id()),
            snap.active_workspace_id(),
            Action::LoginFailed,
            None,
            Some(format!("unlock-failed; remaining={remaining}")),
            now,
        );
        self.audit_writer.record(&event)?;

        let _ = UNLOCK_FAILURE_LIMIT; // 保留对常量的语义引用
        Ok(UnlockOutcome::StillLocked { remaining })
    }
}
