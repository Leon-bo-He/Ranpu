use std::sync::Arc;

use crate::application::ports::{
    AuditWriter, Clock, PasswordHasher, SessionStore, UserRepository,
};

/// Identity 上下文的应用服务。
///
/// 各 use case 通过分散的 `impl IdentityService { ... }` 块（按文件拆分）挂在这里，
/// 由 main.rs composition root 用 Arc<dyn ...> 注入依赖。
#[derive(Clone)]
pub struct IdentityService {
    pub(super) user_repo: Arc<dyn UserRepository>,
    pub(super) hasher: Arc<dyn PasswordHasher>,
    pub(super) audit_writer: Arc<dyn AuditWriter>,
    pub(super) clock: Arc<dyn Clock>,
    pub(super) session_store: Arc<dyn SessionStore>,
}

impl IdentityService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        hasher: Arc<dyn PasswordHasher>,
        audit_writer: Arc<dyn AuditWriter>,
        clock: Arc<dyn Clock>,
        session_store: Arc<dyn SessionStore>,
    ) -> Self {
        Self {
            user_repo,
            hasher,
            audit_writer,
            clock,
            session_store,
        }
    }
}
