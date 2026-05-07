use crate::application::errors::{AppError, AppResult};
use crate::application::identity::service::IdentityService;
use crate::application::ports::errors::RepositoryError;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::identity::errors::IdentityError;
use crate::domain::identity::password::Username;
use crate::domain::identity::role::Role;
use crate::domain::identity::user::User;
use crate::domain::shared::id::UserId;

#[derive(Debug, Clone)]
pub struct CreateUserInput {
    pub username: String,
    pub password: String,
    pub role: Role,
}

const MIN_PASSWORD_LEN: usize = 8;

impl IdentityService {
    /// admin only。第一次启动 (FirstRunSetup) 时由 unauthenticated_create_first_admin
    /// 走专用入口，不经此用例。
    pub fn create_user(&self, input: CreateUserInput) -> AppResult<UserId> {
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
        if input.password.chars().count() < MIN_PASSWORD_LEN {
            return Err(AppError::Identity(IdentityError::PasswordTooShort {
                min: MIN_PASSWORD_LEN,
            }));
        }
        let username = Username::new(input.username.clone())?;
        let hash = self
            .hasher
            .hash(&input.password)
            .map_err(|e| AppError::Crypto(e.to_string()))?;
        let now = self.clock.now();
        let user = User::new(username, hash, input.role, now);
        let id = match self.user_repo.insert(&user) {
            Ok(id) => id,
            Err(RepositoryError::Conflict(_)) => {
                return Err(AppError::Identity(IdentityError::UsernameTaken));
            }
            Err(e) => return Err(AppError::Repository(e)),
        };
        let event = AuditEvent::new(
            Some(snap.user_id()),
            None,
            Action::UserCreated,
            Some(input.username),
            None,
            now,
        );
        self.audit_writer.record(&event)?;
        Ok(id)
    }

    /// 首次启动专用：不需要登录会话，但要求 user 表为空。
    pub fn create_first_admin(&self, input: CreateUserInput) -> AppResult<UserId> {
        if self.user_repo.count()? > 0 {
            return Err(AppError::Internal(
                "已经存在用户，不能重复执行首次启动".to_owned(),
            ));
        }
        if input.password.chars().count() < MIN_PASSWORD_LEN {
            return Err(AppError::Identity(IdentityError::PasswordTooShort {
                min: MIN_PASSWORD_LEN,
            }));
        }
        let username = Username::new(input.username.clone())?;
        let hash = self
            .hasher
            .hash(&input.password)
            .map_err(|e| AppError::Crypto(e.to_string()))?;
        let now = self.clock.now();
        let user = User::new(username, hash, Role::Admin, now);
        let id = self.user_repo.insert(&user)?;
        let event = AuditEvent::new(
            Some(id),
            None,
            Action::UserCreated,
            Some(input.username),
            Some("first-admin".to_owned()),
            now,
        );
        self.audit_writer.record(&event)?;
        Ok(id)
    }
}
