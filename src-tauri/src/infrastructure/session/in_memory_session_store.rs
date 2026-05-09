use parking_lot::Mutex;

use crate::application::ports::session_store::SessionStore;
use crate::domain::session::Session;

/// 进程内单实例的当前会话存储。
pub struct InMemorySessionStore {
    inner: Mutex<Option<Session>>,
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }
}

impl Default for InMemorySessionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionStore for InMemorySessionStore {
    fn current(&self) -> Option<Session> {
        self.inner.lock().clone()
    }

    fn set(&self, session: Session) {
        *self.inner.lock() = Some(session);
    }

    fn clear(&self) {
        *self.inner.lock() = None;
    }

    fn mutate(&self, f: &mut dyn FnMut(&mut Session)) -> bool {
        let mut guard = self.inner.lock();
        match guard.as_mut() {
            Some(s) => {
                f(s);
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn set_then_current_returns_clone() {
        let store = InMemorySessionStore::new();
        let session = Session::new(Utc.timestamp_opt(0, 0).unwrap());
        store.set(session);
        let s = store.current().unwrap();
        assert!(!s.is_locked());
    }

    #[test]
    fn mutate_returns_false_when_empty() {
        let store = InMemorySessionStore::new();
        let mut called = false;
        let modified = store.mutate(&mut |_| called = true);
        assert!(!modified);
        assert!(!called);
    }

    #[test]
    fn mutate_runs_closure_when_present() {
        let store = InMemorySessionStore::new();
        store.set(Session::new(Utc.timestamp_opt(0, 0).unwrap()));
        let modified = store.mutate(&mut |s| s.lock());
        assert!(modified);
        assert!(store.current().unwrap().is_locked());
    }
}
