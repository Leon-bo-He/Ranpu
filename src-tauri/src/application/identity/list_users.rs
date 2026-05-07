use crate::application::errors::{AppError, AppResult};
use crate::application::identity::service::IdentityService;
use crate::domain::identity::user::User;

impl IdentityService {
    pub fn list_users(&self) -> AppResult<Vec<User>> {
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
        Ok(self.user_repo.list_all()?)
    }
}
