//! 通用会话守卫: 取当前会话 + 校验未锁 + (可选) 已激活 workspace.
//!
//! 单用户解锁模型: 没有 ensure_admin 这一层 — 没有用户 / 角色概念,
//! 解锁的人就能做所有事.

use crate::application::errors::{AppError, AppResult};
use crate::application::ports::SessionStore;
use crate::domain::session::Session;
use crate::domain::shared::id::WorkspaceId;

/// 取当前会话; 未解锁或已锁屏返回错误.
pub fn ensure_active(session_store: &dyn SessionStore) -> AppResult<Session> {
    let snap = session_store.current().ok_or(AppError::NotAuthenticated)?;
    if snap.is_locked() {
        return Err(AppError::SessionLocked);
    }
    Ok(snap)
}

/// 取当前会话并要求已激活 workspace.
pub fn ensure_active_workspace(
    session_store: &dyn SessionStore,
) -> AppResult<(Session, WorkspaceId)> {
    let snap = ensure_active(session_store)?;
    let ws = snap
        .active_workspace_id()
        .ok_or(AppError::NoActiveWorkspace)?;
    Ok((snap, ws))
}
