use chrono::{DateTime, Utc};

use crate::application::ports::errors::RepositoryError;
use crate::domain::identity::password::PasswordHash;
use crate::domain::identity::user::User;
use crate::domain::shared::id::UserId;

pub trait UserRepository: Send + Sync {
    fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError>;
    fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError>;
    fn list_all(&self) -> Result<Vec<User>, RepositoryError>;
    /// 给 FirstRunSetup 用：判断是否需要走首次启动流程。
    fn count(&self) -> Result<u64, RepositoryError>;
    /// 用户名冲突应映射为 Conflict。
    fn insert(&self, user: &User) -> Result<UserId, RepositoryError>;
    fn record_failed_attempt(
        &self,
        id: UserId,
        new_failed_attempts: u32,
        locked_until: Option<DateTime<Utc>>,
    ) -> Result<(), RepositoryError>;
    fn mark_successful_login(
        &self,
        id: UserId,
        at: DateTime<Utc>,
    ) -> Result<(), RepositoryError>;
    fn change_password_hash(
        &self,
        id: UserId,
        new_hash: &PasswordHash,
    ) -> Result<(), RepositoryError>;
    fn set_active(&self, id: UserId, is_active: bool) -> Result<(), RepositoryError>;
}
