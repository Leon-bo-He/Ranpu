use chrono::{DateTime, Utc};

use crate::domain::shared::id::WorkspaceId;

/// 应用会话状态 (单用户解锁模型, 仅存内存).
///
/// 没有 user_id / username / role / lockout 计数 — 只有锁屏状态 +
/// 活跃工作区 + 上次活动时间. 解锁失败次数概念也去掉, 用户输错就
/// 再输, 真要防暴力破解走的是 SQLCipher PBKDF2 600k 轮硬算这条路.
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    active_workspace_id: Option<WorkspaceId>,
    locked: bool,
    last_activity_at: DateTime<Utc>,
}

impl Session {
    /// 解锁成功后构造一个新 session.
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            active_workspace_id: None,
            locked: false,
            last_activity_at: now,
        }
    }

    pub fn active_workspace_id(&self) -> Option<WorkspaceId> {
        self.active_workspace_id
    }

    pub fn is_locked(&self) -> bool {
        self.locked
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

    pub fn unlock(&mut self, now: DateTime<Utc>) {
        self.locked = false;
        self.last_activity_at = now;
    }

    /// 当前会话是否需要先选 workspace 才能进行核心操作 (操作工作区配方 / 计算 / 批次清单).
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

    #[test]
    fn new_session_is_unlocked_with_no_workspace() {
        let s = Session::new(t(0));
        assert_eq!(s.active_workspace_id(), None);
        assert!(!s.is_locked());
        assert_eq!(s.last_activity_at(), t(0));
        assert!(s.needs_workspace_for_core_actions());
    }

    #[test]
    fn switch_workspace_round_trip() {
        let mut s = Session::new(t(0));
        s.switch_workspace(Some(WorkspaceId::new(7)));
        assert_eq!(s.active_workspace_id(), Some(WorkspaceId::new(7)));
        s.switch_workspace(None);
        assert_eq!(s.active_workspace_id(), None);
    }

    #[test]
    fn lock_and_unlock_resets_last_activity() {
        let mut s = Session::new(t(0));
        s.lock();
        assert!(s.is_locked());
        s.unlock(t(100));
        assert!(!s.is_locked());
        assert_eq!(s.last_activity_at(), t(100));
    }

    #[test]
    fn record_activity_updates_timestamp() {
        let mut s = Session::new(t(0));
        s.record_activity(t(50));
        assert_eq!(s.last_activity_at(), t(50));
    }
}
