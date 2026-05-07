use std::str::FromStr;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use rusqlite::{params, Row};

use crate::application::ports::errors::RepositoryError;
use crate::application::ports::user_repository::UserRepository;
use crate::domain::identity::password::{PasswordHash, Username};
use crate::domain::identity::role::Role;
use crate::domain::identity::user::User;
use crate::domain::shared::id::UserId;
use crate::infrastructure::persistence::sqlcipher::connection::SqliteConnection;
use crate::infrastructure::persistence::sqlcipher::row_mapping::{
    corrupt, parse_dt, parse_dt_opt, rfc3339,
};

const SELECT_COLS: &str = "id, username, password_hash, role, is_active, failed_attempts, locked_until, created_at, last_login";

pub struct SqliteUserRepository {
    db: Arc<SqliteConnection>,
}

impl SqliteUserRepository {
    pub fn new(db: Arc<SqliteConnection>) -> Self {
        Self { db }
    }
}

/// 把 row 拆成原始字段，再交给 `into_domain` 跑领域重建（隔离 rusqlite::Result
/// 与 RepositoryError，让闭包内部干净）。
struct RawUserRow {
    id: i64,
    username: String,
    password_hash: String,
    role: String,
    is_active: i64,
    failed_attempts: i64,
    locked_until: Option<String>,
    created_at: String,
    last_login: Option<String>,
}

impl RawUserRow {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            username: row.get(1)?,
            password_hash: row.get(2)?,
            role: row.get(3)?,
            is_active: row.get(4)?,
            failed_attempts: row.get(5)?,
            locked_until: row.get(6)?,
            created_at: row.get(7)?,
            last_login: row.get(8)?,
        })
    }

    fn into_domain(self) -> Result<User, RepositoryError> {
        let role = Role::from_str(&self.role).map_err(|e| corrupt("user.role", e))?;
        let username = Username::new(self.username).map_err(|e| corrupt("user.username", e))?;
        let password_hash = PasswordHash::from_phc_string(self.password_hash);
        let locked_until = parse_dt_opt(self.locked_until)?;
        let created_at = parse_dt(&self.created_at)?;
        let last_login = parse_dt_opt(self.last_login)?;
        Ok(User::rehydrate(
            UserId::new(self.id),
            username,
            password_hash,
            role,
            self.is_active != 0,
            self.failed_attempts.max(0) as u32,
            locked_until,
            created_at,
            last_login,
        ))
    }
}

impl UserRepository for SqliteUserRepository {
    fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError> {
        let raw: Option<RawUserRow> = self.db.with(|c| {
            let mut stmt = c.prepare(&format!(
                "SELECT {SELECT_COLS} FROM users WHERE username = ?1"
            ))?;
            let mut rows = stmt.query(params![username])?;
            match rows.next()? {
                None => Ok(None),
                Some(row) => Ok(Some(RawUserRow::from_row(row)?)),
            }
        })?;
        raw.map(|r| r.into_domain()).transpose()
    }

    fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError> {
        let raw: Option<RawUserRow> = self.db.with(|c| {
            let mut stmt = c.prepare(&format!(
                "SELECT {SELECT_COLS} FROM users WHERE id = ?1"
            ))?;
            let mut rows = stmt.query(params![id.value()])?;
            match rows.next()? {
                None => Ok(None),
                Some(row) => Ok(Some(RawUserRow::from_row(row)?)),
            }
        })?;
        raw.map(|r| r.into_domain()).transpose()
    }

    fn list_all(&self) -> Result<Vec<User>, RepositoryError> {
        let raws: Vec<RawUserRow> = self.db.with(|c| {
            let mut stmt = c.prepare(&format!(
                "SELECT {SELECT_COLS} FROM users ORDER BY id"
            ))?;
            let collected: rusqlite::Result<Vec<RawUserRow>> = stmt
                .query_map([], RawUserRow::from_row)?
                .collect();
            collected
        })?;
        raws.into_iter().map(RawUserRow::into_domain).collect()
    }

    fn count(&self) -> Result<u64, RepositoryError> {
        self.db.with(|c| {
            let n: i64 = c.query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))?;
            Ok(n.max(0) as u64)
        })
    }

    fn insert(&self, user: &User) -> Result<UserId, RepositoryError> {
        self.db.with_tx(|tx| {
            tx.execute(
                "INSERT INTO users (username, password_hash, role, is_active, failed_attempts, locked_until, created_at, last_login)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    user.username().as_str(),
                    user.password_hash().as_str(),
                    user.role().as_db_str(),
                    if user.is_active() { 1_i64 } else { 0_i64 },
                    user.failed_attempts() as i64,
                    user.locked_until().map(rfc3339),
                    rfc3339(user.created_at()),
                    user.last_login().map(rfc3339),
                ],
            )?;
            Ok(UserId::new(tx.last_insert_rowid()))
        })
    }

    fn record_failed_attempt(
        &self,
        id: UserId,
        new_failed_attempts: u32,
        locked_until: Option<DateTime<Utc>>,
    ) -> Result<(), RepositoryError> {
        self.db.with(|c| {
            c.execute(
                "UPDATE users SET failed_attempts = ?1, locked_until = ?2 WHERE id = ?3",
                params![
                    new_failed_attempts as i64,
                    locked_until.map(rfc3339),
                    id.value(),
                ],
            )?;
            Ok(())
        })
    }

    fn mark_successful_login(
        &self,
        id: UserId,
        at: DateTime<Utc>,
    ) -> Result<(), RepositoryError> {
        self.db.with(|c| {
            c.execute(
                "UPDATE users SET failed_attempts = 0, locked_until = NULL, last_login = ?1 WHERE id = ?2",
                params![rfc3339(at), id.value()],
            )?;
            Ok(())
        })
    }

    fn change_password_hash(
        &self,
        id: UserId,
        new_hash: &PasswordHash,
    ) -> Result<(), RepositoryError> {
        self.db.with(|c| {
            c.execute(
                "UPDATE users SET password_hash = ?1 WHERE id = ?2",
                params![new_hash.as_str(), id.value()],
            )?;
            Ok(())
        })
    }

    fn set_active(&self, id: UserId, is_active: bool) -> Result<(), RepositoryError> {
        self.db.with(|c| {
            c.execute(
                "UPDATE users SET is_active = ?1 WHERE id = ?2",
                params![if is_active { 1_i64 } else { 0_i64 }, id.value()],
            )?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn db() -> Arc<SqliteConnection> {
        Arc::new(SqliteConnection::open_in_memory().unwrap())
    }

    fn t() -> DateTime<Utc> {
        Utc.timestamp_opt(1_700_000_000, 0).unwrap()
    }

    #[test]
    fn insert_then_find_by_username_returns_persisted_user() {
        let repo = SqliteUserRepository::new(db());
        let user = User::new(
            Username::new("alice").unwrap(),
            PasswordHash::from_phc_string("$argon2id$..."),
            Role::Admin,
            t(),
        );
        let id = repo.insert(&user).unwrap();
        assert!(id.value() > 0);

        let fetched = repo.find_by_username("alice").unwrap().unwrap();
        assert_eq!(fetched.username().as_str(), "alice");
        assert_eq!(fetched.role(), Role::Admin);
        assert!(fetched.is_active());
    }

    #[test]
    fn duplicate_username_is_conflict() {
        let repo = SqliteUserRepository::new(db());
        let u1 = User::new(
            Username::new("alice").unwrap(),
            PasswordHash::from_phc_string("h1"),
            Role::User,
            t(),
        );
        let u2 = User::new(
            Username::new("alice").unwrap(),
            PasswordHash::from_phc_string("h2"),
            Role::User,
            t(),
        );
        repo.insert(&u1).unwrap();
        let err = repo.insert(&u2).unwrap_err();
        assert!(matches!(err, RepositoryError::Conflict(_)));
    }

    #[test]
    fn mark_successful_login_clears_failed_attempts() {
        let repo = SqliteUserRepository::new(db());
        let user = User::new(
            Username::new("alice").unwrap(),
            PasswordHash::from_phc_string("h"),
            Role::User,
            t(),
        );
        let id = repo.insert(&user).unwrap();
        repo.record_failed_attempt(id, 3, None).unwrap();
        repo.mark_successful_login(id, t()).unwrap();
        let fetched = repo.find_by_username("alice").unwrap().unwrap();
        assert_eq!(fetched.failed_attempts(), 0);
        assert!(fetched.locked_until().is_none());
        assert_eq!(fetched.last_login(), Some(t()));
    }

    #[test]
    fn count_increments_with_inserts() {
        let repo = SqliteUserRepository::new(db());
        assert_eq!(repo.count().unwrap(), 0);
        repo.insert(&User::new(
            Username::new("a").unwrap(),
            PasswordHash::from_phc_string("h"),
            Role::User,
            t(),
        ))
        .unwrap();
        assert_eq!(repo.count().unwrap(), 1);
    }
}
