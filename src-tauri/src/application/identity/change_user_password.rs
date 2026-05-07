use crate::application::errors::{AppError, AppResult};
use crate::application::identity::service::IdentityService;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::identity::errors::IdentityError;

#[derive(Debug, Clone)]
pub struct ChangeUserPasswordInput {
    pub old_password: String,
    pub new_password: String,
}

const MIN_PASSWORD_LEN: usize = 8;

impl IdentityService {
    pub fn change_user_password(&self, input: ChangeUserPasswordInput) -> AppResult<()> {
        if input.new_password.is_empty() {
            return Err(AppError::Identity(IdentityError::PasswordEmpty));
        }
        if input.new_password.chars().count() < MIN_PASSWORD_LEN {
            return Err(AppError::Identity(IdentityError::PasswordTooShort {
                min: MIN_PASSWORD_LEN,
            }));
        }

        let snap = self
            .session_store
            .current()
            .ok_or(AppError::NotAuthenticated)?;
        if snap.is_locked() {
            return Err(AppError::SessionLocked);
        }

        let user = self
            .user_repo
            .find_by_id(snap.user_id())?
            .ok_or(AppError::Identity(IdentityError::NotAuthenticated))?;

        let ok = self
            .hasher
            .verify(&input.old_password, user.password_hash())
            .map_err(|e| AppError::Crypto(e.to_string()))?;
        if !ok {
            return Err(AppError::Identity(IdentityError::InvalidCredentials));
        }

        let new_hash = self
            .hasher
            .hash(&input.new_password)
            .map_err(|e| AppError::Crypto(e.to_string()))?;
        self.user_repo.change_password_hash(snap.user_id(), &new_hash)?;

        let event = AuditEvent::new(
            Some(snap.user_id()),
            snap.active_workspace_id(),
            Action::PasswordChanged,
            None,
            None,
            self.clock.now(),
        );
        self.audit_writer.record(&event)?;
        Ok(())
    }
}
