use chrono::{DateTime, Utc};

use crate::domain::identity::password::Username;
use crate::domain::identity::role::Role;
use crate::domain::shared::id::{UserId, WorkspaceId};

/// 解锁连续失败达到该次数即强制登出（PROMPT 第 86 行）。
pub const UNLOCK_FAILURE_LIMIT: u32 = 5;

/// 当前登录会话。
///
/// 仅存于内存（PROMPT 第 87 行：「锁屏不通过后端持久化」）。
/// `active_workspace_id` 在登录后默认为 None；选择 workspace 后才设置。
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    user_id: UserId,
    role: Role,
    username: Username,
    active_workspace_id: Option<WorkspaceId>,
    locked: bool,
    /// 锁屏期间累计的解锁失败次数；解锁成功或重新登录都会清零。
    unlock_failures: u32,
    last_activity_at: DateTime<Utc>,
}

impl Session {
    pub fn new(user_id: UserId, role: Role, username: Username, now: DateTime<Utc>) -> Self {
        Self {
            user_id,
            role,
            username,
            active_workspace_id: None,
            locked: false,
            unlock_failures: 0,
            last_activity_at: now,
        }
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn role(&self) -> Role {
        self.role
    }

    pub fn username(&self) -> &Username {
        &self.username
    }

    pub fn active_workspace_id(&self) -> Option<WorkspaceId> {
        self.active_workspace_id
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn unlock_failures(&self) -> u32 {
        self.unlock_failures
    }

    pub fn last_activity_at(&self) -> DateTime<Utc> {
        self.last_activity_at
    }

    pub fn switch_workspace(&mut self, workspace_id: Option<WorkspaceId>) {
        self.active_workspace_id = workspace_id;
    }

    pub fn record_activity(&mut self, now: DateTime<Utc>) {
        self.last_activity_at = now;
    }

    pub fn lock(&mut self) {
        self.locked = true;
    }

    /// 解锁成功：清除锁定与失败次数，并把活动时间推到 now。
    pub fn unlock(&mut self, now: DateTime<Utc>) {
        self.locked = false;
        self.unlock_failures = 0;
        self.last_activity_at = now;
    }

    /// 解锁失败，返回剩余机会数（0 时调用方应该强制登出）。
    pub fn record_unlock_failure(&mut self) -> u32 {
        self.unlock_failures = self.unlock_failures.saturating_add(1);
        UNLOCK_FAILURE_LIMIT.saturating_sub(self.unlock_failures)
    }

    /// 是否已经触达「强制登出」阈值。
    pub fn should_force_logout(&self) -> bool {
        self.unlock_failures >= UNLOCK_FAILURE_LIMIT
    }

    /// 当前会话是否需要先选 workspace 才能进行核心操作（写配方、计算、用购物车）。
    pub fn needs_workspace_for_core_actions(&self) -> bool {
        self.active_workspace_id.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn t(secs: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(secs, 0).unwrap()
    }

    fn make() -> Session {
        Session::new(
            UserId::new(1),
            Role::User,
            Username::new("bob").unwrap(),
            t(0),
        )
    }

    #[test]
    fn new_session_has_no_active_workspace_and_is_unlocked() {
        let s = make();
        assert_eq!(s.active_workspace_id(), None);
        assert!(!s.is_locked());
        assert_eq!(s.unlock_failures(), 0);
        assert!(s.needs_workspace_for_core_actions());
    }

    #[test]
    fn switch_workspace_sets_then_clears() {
        let mut s = make();
        s.switch_workspace(Some(WorkspaceId::new(7)));
        assert_eq!(s.active_workspace_id(), Some(WorkspaceId::new(7)));
        s.switch_workspace(None);
        assert_eq!(s.active_workspace_id(), None);
    }

    #[test]
    fn lock_then_unlock_clears_state() {
        let mut s = make();
        s.lock();
        assert!(s.is_locked());
        s.record_unlock_failure();
        assert_eq!(s.unlock_failures(), 1);
        s.unlock(t(100));
        assert!(!s.is_locked());
        assert_eq!(s.unlock_failures(), 0);
        assert_eq!(s.last_activity_at(), t(100));
    }

    #[test]
    fn fifth_unlock_failure_signals_force_logout() {
        let mut s = make();
        s.lock();
        for expected_remaining in [4, 3, 2, 1, 0] {
            assert_eq!(s.record_unlock_failure(), expected_remaining);
        }
        assert!(s.should_force_logout());
    }
}
