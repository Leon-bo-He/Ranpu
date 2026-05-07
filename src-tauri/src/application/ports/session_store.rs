use crate::domain::identity::session::Session;

/// 内存中的当前会话存储（PROMPT 第 87 行：锁屏不通过后端持久化）。
pub trait SessionStore: Send + Sync {
    fn current(&self) -> Option<Session>;
    fn set(&self, session: Session);
    fn clear(&self);
    /// 用闭包 in-place 修改当前会话；未登录返回 false（无修改）。
    fn mutate(&self, f: &mut dyn FnMut(&mut Session)) -> bool;
}
