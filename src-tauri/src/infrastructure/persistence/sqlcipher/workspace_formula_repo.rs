use std::sync::Arc;

use rusqlite::{params, params_from_iter, Row};

use crate::application::ports::errors::RepositoryError;
use crate::application::ports::workspace_formula_repository::{
    WorkspaceFormulaQuery, WorkspaceFormulaRepository,
};
use crate::domain::calculation::dye_calculator::CalculableFormula;
use crate::domain::formula::default_formula::DefaultFormula;
use crate::domain::formula::internal_color_code::InternalColorCode;
use crate::domain::formula::workspace_formula::WorkspaceFormula;
use crate::domain::shared::id::{FormulaId, WorkspaceId};
use crate::infrastructure::persistence::sqlcipher::connection::SqliteConnection;
use crate::infrastructure::persistence::sqlcipher::default_formula_repo::replace_items;
use crate::infrastructure::persistence::sqlcipher::row_mapping::{
    corrupt, parse_customer, parse_dt, parse_internal, parse_kg_opt, parse_ratio_opt, parse_unit,
    rfc3339,
};

const COLS: &str = "id, workspace_id, internal_color_code, customer_color_code, color_name, description, base_weight_kg, liquor_ratio, notes, source_default_id, created_at, updated_at";

pub struct SqliteWorkspaceFormulaRepository {
    db: Arc<SqliteConnection>,
}

impl SqliteWorkspaceFormulaRepository {
    pub fn new(db: Arc<SqliteConnection>) -> Self {
        Self { db }
    }
}

struct FormulaRow {
    id: i64,
    workspace_id: i64,
    internal_color_code: String,
    customer_color_code: Option<String>,
    color_name: Option<String>,
    description: Option<String>,
    base_weight_kg: Option<f64>,
    liquor_ratio: Option<f64>,
    notes: Option<String>,
    source_default_id: Option<i64>,
    created_at: String,
    updated_at: String,
}

impl FormulaRow {
    fn from_row(r: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: r.get(0)?,
            workspace_id: r.get(1)?,
            internal_color_code: r.get(2)?,
            customer_color_code: r.get(3)?,
            color_name: r.get(4)?,
            description: r.get(5)?,
            base_weight_kg: r.get(6)?,
            liquor_ratio: r.get(7)?,
            notes: r.get(8)?,
            source_default_id: r.get(9)?,
            created_at: r.get(10)?,
            updated_at: r.get(11)?,
        })
    }
}

struct ItemRow {
    id: i64,
    dye_name: String,
    dye_code: Option<String>,
    percentage: f64,
    unit: String,
    sort_order: i64,
}

impl ItemRow {
    fn from_row(r: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: r.get(0)?,
            dye_name: r.get(1)?,
            dye_code: r.get(2)?,
            percentage: r.get(3)?,
            unit: r.get(4)?,
            sort_order: r.get(5)?,
        })
    }
}

fn fetch_items(
    conn: &rusqlite::Connection,
    formula_id: i64,
) -> rusqlite::Result<Vec<ItemRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, dye_name, dye_code, percentage, unit, sort_order
         FROM workspace_formula_items WHERE formula_id = ?1 ORDER BY sort_order, id",
    )?;
    let collected: rusqlite::Result<Vec<ItemRow>> = stmt
        .query_map(params![formula_id], ItemRow::from_row)?
        .collect();
    collected
}

fn assemble(
    formula: FormulaRow,
    items: Vec<ItemRow>,
) -> Result<WorkspaceFormula, RepositoryError> {
    let internal = parse_internal(formula.internal_color_code)?;
    let customer = parse_customer(formula.customer_color_code)?;
    let base_kg = parse_kg_opt(formula.base_weight_kg)?;
    let ratio = parse_ratio_opt(formula.liquor_ratio)?;
    let created_at = parse_dt(&formula.created_at)?;
    let updated_at = parse_dt(&formula.updated_at)?;
    let mut domain_items = Vec::with_capacity(items.len());
    for it in items {
        let unit = parse_unit(&it.unit)?;
        let item = crate::domain::formula::formula_item::FormulaItem::rehydrate(
            crate::domain::shared::id::FormulaItemId::new(it.id),
            it.dye_name,
            it.dye_code,
            it.percentage,
            unit,
            it.sort_order.clamp(0, u16::MAX as i64) as u16,
        )
        .map_err(|e| corrupt("workspace_formula_item", e))?;
        domain_items.push(item);
    }
    WorkspaceFormula::rehydrate(
        FormulaId::new(formula.id),
        WorkspaceId::new(formula.workspace_id),
        internal,
        customer,
        formula.color_name,
        formula.description,
        base_kg,
        ratio,
        formula.notes,
        domain_items,
        formula.source_default_id.map(FormulaId::new),
        created_at,
        updated_at,
    )
    .map_err(|e| corrupt("workspace_formula", e))
}

impl WorkspaceFormulaRepository for SqliteWorkspaceFormulaRepository {
    fn find_by_id(
        &self,
        workspace_id: WorkspaceId,
        id: FormulaId,
    ) -> Result<Option<WorkspaceFormula>, RepositoryError> {
        let pair: Option<(FormulaRow, Vec<ItemRow>)> = self.db.with(|c| {
            let mut stmt = c.prepare(&format!(
                "SELECT {COLS} FROM workspace_formulas WHERE workspace_id = ?1 AND id = ?2"
            ))?;
            let mut rows = stmt.query(params![workspace_id.value(), id.value()])?;
            match rows.next()? {
                None => Ok(None),
                Some(row) => {
                    let f = FormulaRow::from_row(row)?;
                    let items = fetch_items(c, f.id)?;
                    Ok(Some((f, items)))
                }
            }
        })?;
        pair.map(|(f, items)| assemble(f, items)).transpose()
    }

    fn find_by_internal_code(
        &self,
        workspace_id: WorkspaceId,
        code: &InternalColorCode,
    ) -> Result<Option<WorkspaceFormula>, RepositoryError> {
        let pair: Option<(FormulaRow, Vec<ItemRow>)> = self.db.with(|c| {
            let mut stmt = c.prepare(&format!(
                "SELECT {COLS} FROM workspace_formulas
                 WHERE workspace_id = ?1 AND internal_color_code = ?2"
            ))?;
            let mut rows = stmt.query(params![workspace_id.value(), code.as_str()])?;
            match rows.next()? {
                None => Ok(None),
                Some(row) => {
                    let f = FormulaRow::from_row(row)?;
                    let items = fetch_items(c, f.id)?;
                    Ok(Some((f, items)))
                }
            }
        })?;
        pair.map(|(f, items)| assemble(f, items)).transpose()
    }

    fn find_by_customer_code(
        &self,
        workspace_id: WorkspaceId,
        customer_code: &str,
    ) -> Result<Vec<WorkspaceFormula>, RepositoryError> {
        let pairs: Vec<(FormulaRow, Vec<ItemRow>)> = self.db.with(|c| {
            let mut stmt = c.prepare(&format!(
                "SELECT {COLS} FROM workspace_formulas
                 WHERE workspace_id = ?1 AND customer_color_code = ?2 ORDER BY id"
            ))?;
            let formulas: rusqlite::Result<Vec<FormulaRow>> = stmt
                .query_map(params![workspace_id.value(), customer_code], FormulaRow::from_row)?
                .collect();
            let formulas = formulas?;
            let mut out = Vec::with_capacity(formulas.len());
            for f in formulas {
                let items = fetch_items(c, f.id)?;
                out.push((f, items));
            }
            Ok(out)
        })?;
        pairs.into_iter().map(|(f, items)| assemble(f, items)).collect()
    }

    fn list(
        &self,
        query: WorkspaceFormulaQuery<'_>,
    ) -> Result<Vec<WorkspaceFormula>, RepositoryError> {
        let pairs: Vec<(FormulaRow, Vec<ItemRow>)> = self.db.with(|c| {
            let mut sql = format!(
                "SELECT {COLS} FROM workspace_formulas WHERE workspace_id = ?1"
            );
            let mut bound: Vec<rusqlite::types::Value> = vec![rusqlite::types::Value::Integer(
                query.workspace_id.value(),
            )];
            if let Some(kw) = query.keyword {
                let trimmed = kw.trim();
                if !trimmed.is_empty() {
                    sql.push_str(
                        " AND (internal_color_code LIKE ?2
                              OR customer_color_code LIKE ?2
                              OR color_name LIKE ?2)",
                    );
                    bound.push(rusqlite::types::Value::Text(format!("%{trimmed}%")));
                }
            }
            sql.push_str(" ORDER BY internal_color_code");
            if let Some(limit) = query.limit {
                sql.push_str(&format!(" LIMIT {limit}"));
            }
            if let Some(offset) = query.offset {
                sql.push_str(&format!(" OFFSET {offset}"));
            }
            let mut stmt = c.prepare(&sql)?;
            let formulas: rusqlite::Result<Vec<FormulaRow>> = stmt
                .query_map(params_from_iter(bound.iter()), FormulaRow::from_row)?
                .collect();
            let formulas = formulas?;
            let mut out = Vec::with_capacity(formulas.len());
            for f in formulas {
                let items = fetch_items(c, f.id)?;
                out.push((f, items));
            }
            Ok(out)
        })?;
        pairs.into_iter().map(|(f, items)| assemble(f, items)).collect()
    }

    fn upsert(&self, formula: &WorkspaceFormula) -> Result<FormulaId, RepositoryError> {
        self.db.with_tx(|tx| {
            let id = match formula.id() {
                None => {
                    tx.execute(
                        "INSERT INTO workspace_formulas (workspace_id, internal_color_code,
                            customer_color_code, color_name, description, base_weight_kg,
                            liquor_ratio, notes, source_default_id, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                        params![
                            formula.workspace_id().value(),
                            <WorkspaceFormula as CalculableFormula>::internal_color_code(formula).as_str(),
                            formula.customer_color_code().map(|c| c.as_str()),
                            formula.color_name(),
                            formula.description(),
                            formula.base_weight_kg().map(|k| k.value()),
                            <WorkspaceFormula as CalculableFormula>::liquor_ratio(formula).map(|r| r.value()),
                            formula.notes(),
                            formula.source_default_id().map(|i| i.value()),
                            rfc3339(formula.created_at()),
                            rfc3339(formula.updated_at()),
                        ],
                    )?;
                    tx.last_insert_rowid()
                }
                Some(existing_id) => {
                    tx.execute(
                        "UPDATE workspace_formulas SET
                            internal_color_code = ?1,
                            customer_color_code = ?2,
                            color_name = ?3,
                            description = ?4,
                            base_weight_kg = ?5,
                            liquor_ratio = ?6,
                            notes = ?7,
                            source_default_id = ?8,
                            updated_at = ?9
                         WHERE workspace_id = ?10 AND id = ?11",
                        params![
                            <WorkspaceFormula as CalculableFormula>::internal_color_code(formula).as_str(),
                            formula.customer_color_code().map(|c| c.as_str()),
                            formula.color_name(),
                            formula.description(),
                            formula.base_weight_kg().map(|k| k.value()),
                            <WorkspaceFormula as CalculableFormula>::liquor_ratio(formula).map(|r| r.value()),
                            formula.notes(),
                            formula.source_default_id().map(|i| i.value()),
                            rfc3339(formula.updated_at()),
                            formula.workspace_id().value(),
                            existing_id.value(),
                        ],
                    )?;
                    existing_id.value()
                }
            };

            tx.execute(
                "DELETE FROM workspace_formula_items WHERE formula_id = ?1",
                params![id],
            )?;
            replace_items(
                tx,
                "workspace_formula_items",
                id,
                <WorkspaceFormula as CalculableFormula>::items(formula),
            )?;

            Ok(FormulaId::new(id))
        })
    }

    fn delete(&self, workspace_id: WorkspaceId, id: FormulaId) -> Result<(), RepositoryError> {
        self.db.with(|c| {
            c.execute(
                "DELETE FROM workspace_formulas WHERE workspace_id = ?1 AND id = ?2",
                params![workspace_id.value(), id.value()],
            )?;
            Ok(())
        })
    }

    fn copy_from_default(
        &self,
        default: &DefaultFormula,
        workspace_id: WorkspaceId,
    ) -> Result<FormulaId, RepositoryError> {
        let now = chrono::Utc::now();
        let copied = WorkspaceFormula::new(
            workspace_id,
            <DefaultFormula as CalculableFormula>::internal_color_code(default).clone(),
            default.customer_color_code().cloned(),
            default.color_name().map(str::to_owned),
            default.description().map(str::to_owned),
            default.base_weight_kg(),
            <DefaultFormula as CalculableFormula>::liquor_ratio(default),
            default.notes().map(str::to_owned),
            <DefaultFormula as CalculableFormula>::items(default).to_vec(),
            default.id(),
            now,
        )
        .map_err(|e| RepositoryError::Backend(format!("copy_from_default: {e}")))?;
        self.upsert(&copied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::formula::formula_item::FormulaItem;
    use crate::domain::formula::unit::Unit;
    use chrono::{TimeZone, Utc};

    fn db() -> Arc<SqliteConnection> {
        Arc::new(SqliteConnection::open_in_memory().unwrap())
    }

    fn t() -> chrono::DateTime<Utc> {
        Utc.timestamp_opt(0, 0).unwrap()
    }

    fn ensure_workspace(db: &SqliteConnection) -> WorkspaceId {
        db.with(|c| {
            c.execute(
                "INSERT INTO workspaces (name, created_at) VALUES ('客户A', ?1)",
                params![rfc3339(t())],
            )?;
            Ok(WorkspaceId::new(c.last_insert_rowid()))
        })
        .unwrap()
    }

    fn sample(workspace_id: WorkspaceId) -> WorkspaceFormula {
        WorkspaceFormula::new(
            workspace_id,
            InternalColorCode::new("WK-001").unwrap(),
            None,
            Some("玫红".into()),
            None,
            None,
            None,
            None,
            vec![FormulaItem::new("Reactive Red 195", Some("RR195".into()), 1.0, Unit::PctOwf, 0).unwrap()],
            None,
            t(),
        )
        .unwrap()
    }

    #[test]
    fn insert_and_query_by_internal_code() {
        let db = db();
        let ws = ensure_workspace(&db);
        let repo = SqliteWorkspaceFormulaRepository::new(db);
        let id = repo.upsert(&sample(ws)).unwrap();
        let got = repo
            .find_by_internal_code(ws, &InternalColorCode::new("WK-001").unwrap())
            .unwrap();
        assert_eq!(got.unwrap().id(), Some(id));
    }

    #[test]
    fn duplicate_internal_code_per_workspace_is_conflict() {
        let db = db();
        let ws = ensure_workspace(&db);
        let repo = SqliteWorkspaceFormulaRepository::new(db);
        repo.upsert(&sample(ws)).unwrap();
        let dup = sample(ws);
        let err = repo.upsert(&dup).unwrap_err();
        assert!(matches!(err, RepositoryError::Conflict(_)));
    }
}
