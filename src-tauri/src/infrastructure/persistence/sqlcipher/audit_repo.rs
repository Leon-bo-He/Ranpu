use std::str::FromStr;
use std::sync::Arc;

use rusqlite::{params, params_from_iter, Row};
use uuid::Uuid;

use crate::application::ports::audit_repository::{AuditQuery, AuditRepository};
use crate::application::ports::audit_writer::AuditWriter;
use crate::application::ports::errors::RepositoryError;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::shared::id::{AuditEventId, WorkspaceId};
use crate::infrastructure::persistence::sqlcipher::connection::SqliteConnection;
use crate::infrastructure::persistence::sqlcipher::row_mapping::{corrupt, parse_dt, rfc3339};

const COLS: &str =
    "id, event_uuid, workspace_context_id, action, target, details, occurred_at";

pub struct SqliteAuditRepository {
    db: Arc<SqliteConnection>,
}

impl SqliteAuditRepository {
    pub fn new(db: Arc<SqliteConnection>) -> Self {
        Self { db }
    }
}

struct EventRow {
    id: i64,
    event_uuid: String,
    workspace_context_id: Option<i64>,
    action: String,
    target: Option<String>,
    details: Option<String>,
    occurred_at: String,
}

impl EventRow {
    fn from_row(r: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: r.get(0)?,
            event_uuid: r.get(1)?,
            workspace_context_id: r.get(2)?,
            action: r.get(3)?,
            target: r.get(4)?,
            details: r.get(5)?,
            occurred_at: r.get(6)?,
        })
    }

    fn into_domain(self) -> Result<AuditEvent, RepositoryError> {
        let uuid = Uuid::parse_str(&self.event_uuid).map_err(|e| corrupt("audit.event_uuid", e))?;
        let action = Action::from_str(&self.action).map_err(|e| corrupt("audit.action", e))?;
        let occurred_at = parse_dt(&self.occurred_at)?;
        Ok(AuditEvent::rehydrate(
            AuditEventId::new(self.id),
            uuid,
            self.workspace_context_id.map(WorkspaceId::new),
            action,
            self.target,
            self.details,
            occurred_at,
        ))
    }
}

impl AuditRepository for SqliteAuditRepository {
    fn list(&self, query: AuditQuery<'_>) -> Result<Vec<AuditEvent>, RepositoryError> {
        let raws: Vec<EventRow> = self.db.with(|c| {
            let mut sql = format!("SELECT {COLS} FROM audit_log WHERE 1=1");
            let mut bound: Vec<rusqlite::types::Value> = Vec::new();
            let mut idx = 0_usize;

            if let Some(from) = query.from {
                idx += 1;
                sql.push_str(&format!(" AND occurred_at >= ?{idx}"));
                bound.push(rusqlite::types::Value::Text(rfc3339(from)));
            }
            if let Some(to) = query.to {
                idx += 1;
                sql.push_str(&format!(" AND occurred_at <= ?{idx}"));
                bound.push(rusqlite::types::Value::Text(rfc3339(to)));
            }
            if let Some(actions) = query.actions {
                if !actions.is_empty() {
                    let placeholders: Vec<String> = actions
                        .iter()
                        .map(|_| {
                            idx += 1;
                            format!("?{idx}")
                        })
                        .collect();
                    sql.push_str(&format!(
                        " AND action IN ({})",
                        placeholders.join(",")
                    ));
                    for a in actions {
                        bound.push(rusqlite::types::Value::Text(a.as_db_str().to_owned()));
                    }
                }
            }
            sql.push_str(" ORDER BY occurred_at DESC, id DESC");
            if let Some(limit) = query.limit {
                sql.push_str(&format!(" LIMIT {limit}"));
            }
            if let Some(offset) = query.offset {
                sql.push_str(&format!(" OFFSET {offset}"));
            }

            let mut stmt = c.prepare(&sql)?;
            let collected: rusqlite::Result<Vec<EventRow>> = stmt
                .query_map(params_from_iter(bound.iter()), EventRow::from_row)?
                .collect();
            collected
        })?;
        raws.into_iter().map(EventRow::into_domain).collect()
    }

    fn find_by_id(
        &self,
        id: AuditEventId,
    ) -> Result<Option<AuditEvent>, RepositoryError> {
        let raw: Option<EventRow> = self.db.with(|c| {
            let mut stmt = c.prepare(&format!(
                "SELECT {COLS} FROM audit_log WHERE id = ?1"
            ))?;
            let mut rows = stmt.query(params![id.value()])?;
            match rows.next()? {
                None => Ok(None),
                Some(row) => Ok(Some(EventRow::from_row(row)?)),
            }
        })?;
        raw.map(EventRow::into_domain).transpose()
    }
}

/// Writer 与 Repository 拆分，保持 application 层意图清晰。
pub struct SqliteAuditWriter {
    db: Arc<SqliteConnection>,
}

impl SqliteAuditWriter {
    pub fn new(db: Arc<SqliteConnection>) -> Self {
        Self { db }
    }
}

impl AuditWriter for SqliteAuditWriter {
    fn record(&self, event: &AuditEvent) -> Result<(), RepositoryError> {
        self.db.with(|c| {
            c.execute(
                "INSERT INTO audit_log
                    (event_uuid, workspace_context_id, action, target, details, occurred_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    event.event_uuid().to_string(),
                    event.workspace_context_id().map(|i| i.value()),
                    event.action().as_db_str(),
                    event.target(),
                    event.details(),
                    rfc3339(event.occurred_at()),
                ],
            )?;
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
    fn record_then_list_returns_event() {
        let db = db();
        let writer = SqliteAuditWriter::new(db.clone());
        let repo = SqliteAuditRepository::new(db);

        let event = AuditEvent::new(
            None,
            Action::SessionUnlocked,
            Some("alice".into()),
            None,
            Utc.timestamp_opt(0, 0).unwrap(),
        );
        writer.record(&event).unwrap();

        let listed = repo
            .list(AuditQuery {
                from: None,
                to: None,
                actions: None,
                limit: None,
                offset: None,
            })
            .unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].action(), Action::SessionUnlocked);
    }

    #[test]
    fn list_filter_by_action_works() {
        let db = db();
        let writer = SqliteAuditWriter::new(db.clone());
        let repo = SqliteAuditRepository::new(db);
        for action in [Action::SessionUnlocked, Action::CartItemAdded, Action::CalculationPerformed]
        {
            writer
                .record(&AuditEvent::new(
                    None,
                    action,
                    None,
                    None,
                    Utc.timestamp_opt(0, 0).unwrap(),
                ))
                .unwrap();
        }
        let only_calc = repo
            .list(AuditQuery {
                from: None,
                to: None,
                actions: Some(&[Action::CalculationPerformed]),
                limit: None,
                offset: None,
            })
            .unwrap();
        assert_eq!(only_calc.len(), 1);
        assert_eq!(only_calc[0].action(), Action::CalculationPerformed);
    }
}
