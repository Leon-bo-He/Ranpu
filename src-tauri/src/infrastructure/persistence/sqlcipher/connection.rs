use std::path::Path;
use std::sync::Arc;

use parking_lot::Mutex;
use rusqlite::{Connection, OpenFlags};

use crate::application::ports::errors::RepositoryError;

const SCHEMA_SQL: &str = include_str!("../schema.sql");

/// 包装好的 SQLCipher 连接：单连接 + Mutex，足够桌面单用户场景。
///
/// PROMPT 加密设计：PRAGMA key 由「主密钥 + 启动口令」PBKDF2 派生 64 hex
/// （feat/infra-crypto 实装），本层只负责把派生好的 key 喂给 SQLCipher。
pub struct SqliteConnection {
    inner: Arc<Mutex<Connection>>,
}

impl SqliteConnection {
    /// 打开 / 创建数据库。`key_hex` 必须是 64 字符（32 字节）的 16 进制字符串。
    pub fn open(path: &Path, key_hex: &str) -> Result<Self, RepositoryError> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )
        .map_err(map_err)?;

        // 必须在第一条 SQL 之前。错误的 key 会让后续 SELECT 报 NotADatabase。
        conn.pragma_update(None, "key", format!("x'{key_hex}'"))
            .map_err(map_err)?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(map_err)?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(map_err)?;
        conn.execute_batch(SCHEMA_SQL).map_err(map_err)?;
        run_migrations(&conn).map_err(map_err)?;

        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    /// 仅供测试：建一个内存数据库（无加密），仍套上同一份 schema。
    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, RepositoryError> {
        let conn = Connection::open_in_memory().map_err(map_err)?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(map_err)?;
        conn.execute_batch(SCHEMA_SQL).map_err(map_err)?;
        run_migrations(&conn).map_err(map_err)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn with<F, T>(&self, f: F) -> Result<T, RepositoryError>
    where
        F: FnOnce(&Connection) -> rusqlite::Result<T>,
    {
        let guard = self.inner.lock();
        f(&guard).map_err(map_err)
    }

    pub fn with_tx<F, T>(&self, f: F) -> Result<T, RepositoryError>
    where
        F: FnOnce(&rusqlite::Transaction<'_>) -> rusqlite::Result<T>,
    {
        let mut guard = self.inner.lock();
        let tx = guard.transaction().map_err(map_err)?;
        let out = f(&tx).map_err(map_err)?;
        tx.commit().map_err(map_err)?;
        Ok(out)
    }

    pub fn arc(&self) -> Arc<Mutex<Connection>> {
        self.inner.clone()
    }
}

/// 增量迁移: 升级老版本 DB 的 schema (CREATE TABLE IF NOT EXISTS 不会动已存在的表).
///
/// 全部用 PRAGMA table_info 探测列是否存在; 不存在就 ALTER TABLE ADD COLUMN.
/// 必须幂等, 启动每次都跑.
fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    if !column_exists(conn, "workspaces", "kind")? {
        conn.execute_batch(
            "ALTER TABLE workspaces ADD COLUMN kind TEXT NOT NULL DEFAULT 'normal'",
        )?;
    }
    Ok(())
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> rusqlite::Result<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == column {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(crate) fn map_err(e: rusqlite::Error) -> RepositoryError {
    use rusqlite::ffi::ErrorCode;
    use rusqlite::Error;
    match &e {
        Error::SqliteFailure(err, msg)
            if matches!(
                err.code,
                ErrorCode::ConstraintViolation | ErrorCode::DatabaseCorrupt
            ) =>
        {
            let detail = msg
                .clone()
                .unwrap_or_else(|| format!("constraint violation: {e}"));
            RepositoryError::Conflict(detail)
        }
        Error::QueryReturnedNoRows => RepositoryError::NotFound,
        _ => RepositoryError::Backend(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_db_runs_schema_and_has_users_table() {
        let db = SqliteConnection::open_in_memory().unwrap();
        let count: i64 = db
            .with(|c| {
                c.query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='users'",
                    [],
                    |r| r.get(0),
                )
            })
            .unwrap();
        assert_eq!(count, 1);
    }
}
