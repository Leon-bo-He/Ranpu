use std::str::FromStr;
use std::sync::Arc;

use rusqlite::{params, Row};

use crate::application::ports::errors::RepositoryError;
use crate::application::ports::workspace_repository::WorkspaceRepository;
use crate::domain::shared::id::{UserId, WorkspaceId};
use crate::domain::workspace::workspace::{Workspace, WorkspaceKind, WorkspaceName};
use crate::infrastructure::persistence::sqlcipher::connection::SqliteConnection;
use crate::infrastructure::persistence::sqlcipher::row_mapping::{corrupt, parse_dt, rfc3339};

const SELECT_COLS: &str = "id, name, description, created_by_user_id, created_at, kind";

pub struct SqliteWorkspaceRepository {
    db: Arc<SqliteConnection>,
}

impl SqliteWorkspaceRepository {
    pub fn new(db: Arc<SqliteConnection>) -> Self {
        Self { db }
    }
}

struct RawRow {
    id: i64,
    name: String,
    description: Option<String>,
    created_by_user_id: Option<i64>,
    created_at: String,
    kind: String,
}

impl RawRow {
    fn from_row(r: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: r.get(0)?,
            name: r.get(1)?,
            description: r.get(2)?,
            created_by_user_id: r.get(3)?,
            created_at: r.get(4)?,
            kind: r.get(5)?,
        })
    }

    fn into_domain(self) -> Result<Workspace, RepositoryError> {
        let name = WorkspaceName::new(self.name).map_err(|e| corrupt("workspace.name", e))?;
        let created_at = parse_dt(&self.created_at)?;
        let kind =
            WorkspaceKind::from_str(&self.kind).map_err(|e| corrupt("workspace.kind", e))?;
        Ok(Workspace::rehydrate_with_kind(
            WorkspaceId::new(self.id),
            name,
            self.description,
            self.created_by_user_id.map(UserId::new),
            created_at,
            kind,
        ))
    }
}

impl WorkspaceRepository for SqliteWorkspaceRepository {
    fn find_by_id(&self, id: WorkspaceId) -> Result<Option<Workspace>, RepositoryError> {
        let raw: Option<RawRow> = self.db.with(|c| {
            let mut stmt = c.prepare(&format!(
                "SELECT {SELECT_COLS} FROM workspaces WHERE id = ?1"
            ))?;
            let mut rows = stmt.query(params![id.value()])?;
            match rows.next()? {
                None => Ok(None),
                Some(row) => Ok(Some(RawRow::from_row(row)?)),
            }
        })?;
        raw.map(RawRow::into_domain).transpose()
    }

    fn find_by_name(&self, name: &str) -> Result<Option<Workspace>, RepositoryError> {
        let raw: Option<RawRow> = self.db.with(|c| {
            let mut stmt = c.prepare(&format!(
                "SELECT {SELECT_COLS} FROM workspaces WHERE name = ?1"
            ))?;
            let mut rows = stmt.query(params![name])?;
            match rows.next()? {
                None => Ok(None),
                Some(row) => Ok(Some(RawRow::from_row(row)?)),
            }
        })?;
        raw.map(RawRow::into_domain).transpose()
    }

    fn list_all(&self) -> Result<Vec<Workspace>, RepositoryError> {
        let raws: Vec<RawRow> = self.db.with(|c| {
            let mut stmt = c.prepare(&format!(
                "SELECT {SELECT_COLS} FROM workspaces ORDER BY id"
            ))?;
            let collected: rusqlite::Result<Vec<RawRow>> = stmt
                .query_map([], RawRow::from_row)?
                .collect();
            collected
        })?;
        raws.into_iter().map(RawRow::into_domain).collect()
    }

    fn find_system_mirror(&self) -> Result<Option<Workspace>, RepositoryError> {
        let raw: Option<RawRow> = self.db.with(|c| {
            let mut stmt = c.prepare(&format!(
                "SELECT {SELECT_COLS} FROM workspaces WHERE kind = 'system_mirror' ORDER BY id LIMIT 1"
            ))?;
            let mut rows = stmt.query([])?;
            match rows.next()? {
                None => Ok(None),
                Some(row) => Ok(Some(RawRow::from_row(row)?)),
            }
        })?;
        raw.map(RawRow::into_domain).transpose()
    }

    fn insert(&self, workspace: &Workspace) -> Result<WorkspaceId, RepositoryError> {
        self.db.with_tx(|tx| {
            tx.execute(
                "INSERT INTO workspaces (name, description, created_by_user_id, created_at, kind) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    workspace.name().as_str(),
                    workspace.description(),
                    workspace.created_by_user_id().map(|i| i.value()),
                    rfc3339(workspace.created_at()),
                    workspace.kind().as_db_str(),
                ],
            )?;
            Ok(WorkspaceId::new(tx.last_insert_rowid()))
        })
    }

    fn rename(&self, id: WorkspaceId, new_name: &str) -> Result<(), RepositoryError> {
        self.db.with(|c| {
            c.execute(
                "UPDATE workspaces SET name = ?1 WHERE id = ?2",
                params![new_name, id.value()],
            )?;
            Ok(())
        })
    }

    fn update_description(
        &self,
        id: WorkspaceId,
        description: Option<&str>,
    ) -> Result<(), RepositoryError> {
        self.db.with(|c| {
            c.execute(
                "UPDATE workspaces SET description = ?1 WHERE id = ?2",
                params![description, id.value()],
            )?;
            Ok(())
        })
    }

    fn delete(&self, id: WorkspaceId) -> Result<(), RepositoryError> {
        self.db.with(|c| {
            c.execute("DELETE FROM workspaces WHERE id = ?1", params![id.value()])?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn db() -> Arc<SqliteConnection> {
        Arc::new(SqliteConnection::open_in_memory().unwrap())
    }

    #[test]
    fn round_trip_workspace() {
        let repo = SqliteWorkspaceRepository::new(db());
        let now = Utc.timestamp_opt(0, 0).unwrap();
        let w = Workspace::new(
            WorkspaceName::new("客户A").unwrap(),
            Some("第一个客户".into()),
            None,
            now,
        )
        .unwrap();
        let id = repo.insert(&w).unwrap();
        let got = repo.find_by_id(id).unwrap().unwrap();
        assert_eq!(got.name().as_str(), "客户A");
        assert_eq!(got.description(), Some("第一个客户"));
        assert_eq!(got.kind(), WorkspaceKind::Normal);
    }

    #[test]
    fn round_trip_system_mirror_kind() {
        let repo = SqliteWorkspaceRepository::new(db());
        let now = Utc.timestamp_opt(0, 0).unwrap();
        let w = Workspace::new_with_kind(
            WorkspaceName::new("通用").unwrap(),
            None,
            None,
            now,
            WorkspaceKind::SystemMirror,
        )
        .unwrap();
        let id = repo.insert(&w).unwrap();
        let got = repo.find_by_id(id).unwrap().unwrap();
        assert!(got.is_system_mirror());
    }

    #[test]
    fn duplicate_name_conflict() {
        let repo = SqliteWorkspaceRepository::new(db());
        let now = Utc.timestamp_opt(0, 0).unwrap();
        let a = Workspace::new(WorkspaceName::new("X").unwrap(), None, None, now).unwrap();
        repo.insert(&a).unwrap();
        let b = Workspace::new(WorkspaceName::new("X").unwrap(), None, None, now).unwrap();
        assert!(matches!(repo.insert(&b), Err(RepositoryError::Conflict(_))));
    }
}
