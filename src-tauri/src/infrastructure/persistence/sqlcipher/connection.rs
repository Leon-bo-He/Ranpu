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

    // 单用户迁移: 老 DB 有 users 表 + user_id / created_by_user_id FK; 全部清掉.
    // SQLCipher / SQLite ≥ 3.35 支持 ALTER TABLE DROP COLUMN. cart_items 因 UNIQUE
    // 跨 user_id, 用 "新建表 + 拷数据 + 改名" 的标准模式重建.
    if column_exists(conn, "workspaces", "created_by_user_id")? {
        conn.execute_batch("ALTER TABLE workspaces DROP COLUMN created_by_user_id")?;
    }
    if column_exists(conn, "default_formulas", "created_by_user_id")? {
        conn.execute_batch("ALTER TABLE default_formulas DROP COLUMN created_by_user_id")?;
    }
    if column_exists(conn, "audit_log", "user_id")? {
        conn.execute_batch("ALTER TABLE audit_log DROP COLUMN user_id")?;
    }
    if column_exists(conn, "cart_items", "user_id")? {
        // UNIQUE 跨 user_id, 直接 DROP COLUMN 会被 SQLite 拒绝. 整张表重建.
        conn.execute_batch(
            "CREATE TABLE cart_items_new (
                id                INTEGER PRIMARY KEY AUTOINCREMENT,
                workspace_id      INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
                source_kind       TEXT NOT NULL CHECK(source_kind IN ('default','workspace')),
                source_formula_id INTEGER NOT NULL,
                target_kg         REAL NOT NULL CHECK(target_kg > 0),
                added_at          TEXT NOT NULL,
                UNIQUE(workspace_id, source_kind, source_formula_id)
            );
            INSERT OR IGNORE INTO cart_items_new
                (id, workspace_id, source_kind, source_formula_id, target_kg, added_at)
            SELECT id, workspace_id, source_kind, source_formula_id, target_kg, added_at
            FROM cart_items;
            DROP TABLE cart_items;
            ALTER TABLE cart_items_new RENAME TO cart_items;
            CREATE INDEX IF NOT EXISTS idx_cart_workspace ON cart_items(workspace_id);",
        )?;
    }
    if table_exists(conn, "users")? {
        conn.execute_batch("DROP TABLE users")?;
    }

    // 老索引按 user_id 建的, 现在 user_id 没了, 索引也失效. 删掉重建.
    let _ = conn.execute_batch("DROP INDEX IF EXISTS idx_cart_user_workspace");
    let _ = conn.execute_batch("DROP INDEX IF EXISTS idx_audit_user_time");

    // 配方字段精简: 砍 color_name / description / base_weight_kg / liquor_ratio,
    // 加 color_family. SQLite ≥ 3.35 ALTER TABLE DROP COLUMN. items 表 unit
    // 列的 CHECK 约束改了 (去掉 g_per_L), 但 SQLite CHECK 改约束要重建表;
    // 不重建也行 — 现存行如果有 g_per_L 留着就好, 之后写入会触发新约束.
    // 实际上 g_per_L 的行会在重建过程中变成不合法; 直接整表重建更稳.
    for table in ["default_formulas", "workspace_formulas"] {
        if column_exists(conn, table, "color_name")? {
            conn.execute_batch(&format!("ALTER TABLE {table} DROP COLUMN color_name"))?;
        }
        if column_exists(conn, table, "description")? {
            conn.execute_batch(&format!("ALTER TABLE {table} DROP COLUMN description"))?;
        }
        if column_exists(conn, table, "base_weight_kg")? {
            conn.execute_batch(&format!("ALTER TABLE {table} DROP COLUMN base_weight_kg"))?;
        }
        if column_exists(conn, table, "liquor_ratio")? {
            conn.execute_batch(&format!("ALTER TABLE {table} DROP COLUMN liquor_ratio"))?;
        }
        if !column_exists(conn, table, "color_family")? {
            conn.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN color_family TEXT"))?;
        }
    }

    // color_family 索引在这里建 (而不是 schema.sql), 因为老 DB 上 schema.sql
    // 那段先于 ALTER TABLE ADD COLUMN 跑, 直接 CREATE INDEX(color_family) 会
    // "no such column" 报错. 现在 ADD COLUMN 已完成, 这里 CREATE INDEX 安全.
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_workspace_formulas_ws_family
            ON workspace_formulas(workspace_id, color_family);
         CREATE INDEX IF NOT EXISTS idx_default_formulas_family
            ON default_formulas(color_family);",
    )?;

    // 老 items 表 unit CHECK 还允许 g_per_L. 数据里如果有 g_per_L 行先迁成 g_per_kg
    // (语义上的近似 fallback, 总比直接报错强), 然后整表重建以同步新 CHECK.
    for items_table in ["default_formula_items", "workspace_formula_items"] {
        let needs_unit_rebuild = check_clause_has_g_per_l(conn, items_table)?;
        if needs_unit_rebuild {
            conn.execute_batch(&format!(
                "UPDATE {items_table} SET unit = 'g_per_kg' WHERE unit = 'g_per_L';
                 CREATE TABLE {items_table}_new (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    formula_id  INTEGER NOT NULL,
                    dye_name    TEXT NOT NULL,
                    dye_code    TEXT,
                    percentage  REAL NOT NULL,
                    unit        TEXT NOT NULL CHECK(unit IN ('pct_owf','g_per_kg')),
                    sort_order  INTEGER NOT NULL DEFAULT 0
                 );
                 INSERT INTO {items_table}_new
                    (id, formula_id, dye_name, dye_code, percentage, unit, sort_order)
                 SELECT id, formula_id, dye_name, dye_code, percentage, unit, sort_order
                 FROM {items_table};
                 DROP TABLE {items_table};
                 ALTER TABLE {items_table}_new RENAME TO {items_table};"
            ))?;
        }
    }

    Ok(())
}

/// items 表 unit 列的 CHECK 约束是否还允许 g_per_L (老 schema). PRAGMA
/// table_info 不暴露 CHECK 详情, 走 sqlite_master 抓建表语句字符串.
fn check_clause_has_g_per_l(conn: &Connection, table: &str) -> rusqlite::Result<bool> {
    let sql: Option<String> = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name=?",
            [table],
            |r| r.get(0),
        )
        .ok();
    Ok(sql
        .map(|s| s.contains("g_per_L"))
        .unwrap_or(false))
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

fn table_exists(conn: &Connection, table: &str) -> rusqlite::Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?",
        [table],
        |r| r.get(0),
    )?;
    Ok(count > 0)
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
    fn in_memory_db_runs_schema_and_has_workspaces_table() {
        let db = SqliteConnection::open_in_memory().unwrap();
        let count: i64 = db
            .with(|c| {
                c.query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='workspaces'",
                    [],
                    |r| r.get(0),
                )
            })
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn in_memory_db_has_no_users_table() {
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
        assert_eq!(count, 0);
    }
}
