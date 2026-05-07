use crate::application::errors::{AppError, AppResult};
use crate::application::identity::service::IdentityService;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::identity::errors::IdentityError;
use crate::domain::identity::session::Session;
use crate::domain::identity::user::User;

#[derive(Debug, Clone)]
pub struct AuthenticateUserInput {
    pub username: String,
    pub password: String,
}

impl IdentityService {
    /// 登录用例。
    ///
    /// 防枚举：用户名不存在与密码错都映射为同一类错（仅在剩余次数提示上有差异）。
    pub fn authenticate_user(&self, input: AuthenticateUserInput) -> AppResult<Session> {
        let now = self.clock.now();

        // 1. 取用户。不存在 → 防枚举的统一错误。
        let mut user: User = match self.user_repo.find_by_username(&input.username)? {
            Some(u) => u,
            None => {
                self.write_audit(None, Action::LoginFailed, Some(input.username.clone()))?;
                return Err(AppError::Identity(IdentityError::InvalidCredentials));
            }
        };

        let user_id = user.id().expect("仓储层必须返回带 id 的 User");

        // 2. 是否允许尝试登录？
        if let Err(e) = user.ensure_can_attempt_login(now) {
            self.write_audit(Some(user_id), Action::LoginFailed, Some(input.username.clone()))?;
            return Err(AppError::Identity(e));
        }

        // 3. 校验密码。
        let password_ok = self
            .hasher
            .verify(&input.password, user.password_hash())
            .map_err(|e| AppError::Crypto(e.to_string()))?;

        if !password_ok {
            // 记录失败、可能触发锁定。
            let err = user.record_failed_login(now);
            self.user_repo.record_failed_attempt(
                user_id,
                user.failed_attempts(),
                user.locked_until(),
            )?;
            let action = match &err {
                IdentityError::AccountJustLocked { .. } => Action::AccountLocked,
                _ => Action::LoginFailed,
            };
            self.write_audit(Some(user_id), action, Some(input.username.clone()))?;
            return Err(AppError::Identity(err));
        }

        // 4. 登录成功：刷新仓储 + 写审计 + 建会话。
        user.mark_successful_login(now);
        self.user_repo.mark_successful_login(user_id, now)?;
        self.write_audit(
            Some(user_id),
            Action::LoginSucceeded,
            Some(input.username.clone()),
        )?;

        let session = Session::new(user_id, user.role(), user.username().clone(), now);
        self.session_store.set(session.clone());
        Ok(session)
    }

    fn write_audit(
        &self,
        user_id: Option<crate::domain::shared::id::UserId>,
        action: Action,
        target: Option<String>,
    ) -> AppResult<()> {
        let event = AuditEvent::new(user_id, None, action, target, None, self.clock.now());
        self.audit_writer.record(&event)?;
        Ok(())
    }
}
