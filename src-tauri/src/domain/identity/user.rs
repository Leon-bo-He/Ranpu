use chrono::{DateTime, Duration, Utc};

use crate::domain::identity::errors::IdentityError;
use crate::domain::identity::password::{PasswordHash, Username};
use crate::domain::identity::role::Role;
use crate::domain::shared::id::UserId;

/// 连续登录失败达到该次数即触发锁定（PROMPT 第 60 行）。
pub const LOCKOUT_THRESHOLD: u32 = 5;
/// 触发锁定后的锁定时长。
pub const LOCKOUT_DURATION_MINUTES: i64 = 15;

/// User 聚合根。
///
/// 保存的密码必然是 argon2 哈希；登录失败计数与锁定时间也都是聚合状态，
/// 通过下面的方法变更（不要从外部直接修改字段）。`id` 在持久化前为 None。
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    id: Option<UserId>,
    username: Username,
    password_hash: PasswordHash,
    role: Role,
    is_active: bool,
    failed_attempts: u32,
    locked_until: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    last_login: Option<DateTime<Utc>>,
}

impl User {
    /// 创建一个尚未持久化的新用户（仓储 insert 后会回填 id）。
    pub fn new(
        username: Username,
        password_hash: PasswordHash,
        role: Role,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: None,
            username,
            password_hash,
            role,
            is_active: true,
            failed_attempts: 0,
            locked_until: None,
            created_at,
            last_login: None,
        }
    }

    /// 从仓储重建一个已持久化的 User。仓储层调用，不做业务校验。
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: UserId,
        username: Username,
        password_hash: PasswordHash,
        role: Role,
        is_active: bool,
        failed_attempts: u32,
        locked_until: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        last_login: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: Some(id),
            username,
            password_hash,
            role,
            is_active,
            failed_attempts,
            locked_until,
            created_at,
            last_login,
        }
    }

    pub fn id(&self) -> Option<UserId> {
        self.id
    }

    pub fn username(&self) -> &Username {
        &self.username
    }

    pub fn password_hash(&self) -> &PasswordHash {
        &self.password_hash
    }

    pub fn role(&self) -> Role {
        self.role
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn failed_attempts(&self) -> u32 {
        self.failed_attempts
    }

    pub fn locked_until(&self) -> Option<DateTime<Utc>> {
        self.locked_until
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn last_login(&self) -> Option<DateTime<Utc>> {
        self.last_login
    }

    pub fn assign_id(&mut self, id: UserId) {
        self.id = Some(id);
    }

    /// 当前是否处于锁定中。锁定到期会自动恢复，调用方传入「现在」用以判断。
    pub fn is_locked_at(&self, now: DateTime<Utc>) -> bool {
        match self.locked_until {
            Some(until) => until > now,
            None => false,
        }
    }

    /// 已停用 + 已锁定 都属于「不可登录」状态。
    pub fn ensure_can_attempt_login(&self, now: DateTime<Utc>) -> Result<(), IdentityError> {
        if !self.is_active {
            return Err(IdentityError::AccountInactive);
        }
        if let Some(until) = self.locked_until {
            if until > now {
                return Err(IdentityError::AccountLocked { until });
            }
        }
        Ok(())
    }

    /// 记一次登录失败，并返回应该向 UI 显示的错误。
    /// 第 5 次失败时同时设置 locked_until = now + 15min。
    pub fn record_failed_login(&mut self, now: DateTime<Utc>) -> IdentityError {
        self.failed_attempts = self.failed_attempts.saturating_add(1);
        if self.failed_attempts >= LOCKOUT_THRESHOLD {
            let until = now + Duration::minutes(LOCKOUT_DURATION_MINUTES);
            self.locked_until = Some(until);
            IdentityError::AccountJustLocked { until }
        } else {
            let remaining = LOCKOUT_THRESHOLD - self.failed_attempts;
            IdentityError::InvalidCredentialsWithRemaining { remaining }
        }
    }

    /// 登录成功：清空 failed_attempts、解锁、写 last_login。
    pub fn mark_successful_login(&mut self, now: DateTime<Utc>) {
        self.failed_attempts = 0;
        self.locked_until = None;
        self.last_login = Some(now);
    }

    /// 锁定到期后由仓储层调用，把 locked_until 清掉。
    pub fn clear_expired_lock(&mut self, now: DateTime<Utc>) -> bool {
        if let Some(until) = self.locked_until {
            if until <= now {
                self.locked_until = None;
                self.failed_attempts = 0;
                return true;
            }
        }
        false
    }

    pub fn change_password_hash(&mut self, new_hash: PasswordHash) {
        self.password_hash = new_hash;
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn activate(&mut self) {
        self.is_active = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn t(secs: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(secs, 0).unwrap()
    }

    fn make_user() -> User {
        User::new(
            Username::new("alice").unwrap(),
            PasswordHash::from_phc_string("$argon2id$..."),
            Role::User,
            t(1_000_000),
        )
    }

    #[test]
    fn new_user_has_no_id_and_zero_failures() {
        let u = make_user();
        assert!(u.id().is_none());
        assert_eq!(u.failed_attempts(), 0);
        assert!(u.is_active());
        assert!(u.locked_until().is_none());
    }

    #[test]
    fn record_failed_login_returns_remaining_until_threshold() {
        let mut u = make_user();
        for expected_remaining in (1..=4).rev() {
            let err = u.record_failed_login(t(1_000_100));
            assert!(matches!(
                err,
                IdentityError::InvalidCredentialsWithRemaining { remaining } if remaining == expected_remaining
            ));
        }
    }

    #[test]
    fn fifth_failure_locks_account_for_15_minutes() {
        let mut u = make_user();
        for _ in 0..4 {
            u.record_failed_login(t(1_000_100));
        }
        let err = u.record_failed_login(t(1_000_200));
        let until = match err {
            IdentityError::AccountJustLocked { until } => until,
            other => panic!("unexpected: {other:?}"),
        };
        assert_eq!(until, t(1_000_200) + Duration::minutes(15));
        assert!(u.is_locked_at(t(1_000_200)));
        assert!(u.is_locked_at(t(1_000_200) + Duration::minutes(14)));
        assert!(!u.is_locked_at(t(1_000_200) + Duration::minutes(16)));
    }

    #[test]
    fn ensure_can_attempt_returns_account_locked_while_within_window() {
        let mut u = make_user();
        for _ in 0..5 {
            u.record_failed_login(t(0));
        }
        assert!(matches!(
            u.ensure_can_attempt_login(t(60)),
            Err(IdentityError::AccountLocked { .. })
        ));
    }

    #[test]
    fn successful_login_clears_failures_and_lock() {
        let mut u = make_user();
        u.record_failed_login(t(0));
        u.record_failed_login(t(0));
        u.mark_successful_login(t(100));
        assert_eq!(u.failed_attempts(), 0);
        assert!(u.locked_until().is_none());
        assert_eq!(u.last_login(), Some(t(100)));
    }

    #[test]
    fn deactivated_user_cannot_attempt_login() {
        let mut u = make_user();
        u.deactivate();
        assert!(matches!(
            u.ensure_can_attempt_login(t(0)),
            Err(IdentityError::AccountInactive)
        ));
    }

    #[test]
    fn clear_expired_lock_resets_failures() {
        let mut u = make_user();
        for _ in 0..5 {
            u.record_failed_login(t(0));
        }
        assert!(u.clear_expired_lock(t(0) + Duration::minutes(20)));
        assert!(u.locked_until().is_none());
        assert_eq!(u.failed_attempts(), 0);
    }
}
