use std::sync::Arc;

use rusqlite::{params, params_from_iter, Row, Transaction};

use crate::application::ports::default_formula_repository::{
    DefaultFormulaQuery, DefaultFormulaRepository,
};
use crate::application::ports::errors::RepositoryError;
use crate::domain::formula::default_formula::DefaultFormula;
use crate::domain::formula::formula_item::FormulaItem;
use crate::domain::formula::internal_color_code::InternalColorCode;
use crate::domain::shared::id::{FormulaId, FormulaItemId};
use crate::infrastructure::persistence::sqlcipher::connection::SqliteConnection;
use crate::infrastructure::persistence::sqlcipher::row_mapping::{
    corrupt, parse_customer, parse_dt, parse_internal, parse_kg_opt, parse_ratio_opt, parse_unit,
    rfc3339,
};

const COLS: &str = "id, internal_color_code, customer_color_code, color_name, description, base_weight_kg, liquor_ratio, notes, created_at, updated_at";

pub struct SqliteDefaultFormulaRepository {
    db: Arc<SqliteConnection>,
}

impl SqliteDefaultFormulaRepository {
    pub fn new(db: Arc<SqliteConnection>) -> Self {
        Self { db }
    }
}

struct FormulaRow {
    id: i64,
    internal_color_code: String,
    customer_color_code: Option<String>,
    color_name: Option<String>,
    description: Option<String>,
    base_weight_kg: Option<f64>,
    liquor_ratio: Option<f64>,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
}

impl FormulaRow {
    fn from_row(r: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: r.get(0)?,
            internal_color_code: r.get(1)?,
            customer_color_code: r.get(2)?,
            color_name: r.get(3)?,
            description: r.get(4)?,
            base_weight_kg: r.get(5)?,
            liquor_ratio: r.get(6)?,
            notes: r.get(7)?,
            created_at: r.get(8)?,
            updated_at: r.get(9)?,
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

    fn into_domain(self) -> Result<FormulaItem, RepositoryError> {
        let unit = parse_unit(&self.unit)?;
        FormulaItem::rehydrate(
            FormulaItemId::new(self.id),
            self.dye_name,
            self.dye_code,
            self.percentage,
            unit,
            self.sort_order.clamp(0, u16::MAX as i64) as u16,
        )
        .map_err(|e| corrupt("formula_item", e))
    }
}

fn fetch_items(
    tx: &rusqlite::Connection,
    table: &str,
    formula_id: i64,
) -> rusqlite::Result<Vec<ItemRow>> {
    let mut stmt = tx.prepare(&format!(
        "SELECT id, dye_name, dye_code, percentage, unit, sort_order
         FROM {table} WHERE formula_id = ?1 ORDER BY sort_order, id"
    ))?;
    let collected: rusqlite::Result<Vec<ItemRow>> = stmt
        .query_map(params![formula_id], ItemRow::from_row)?
        .collect();
    collected
}

fn assemble_default(
    formula_row: FormulaRow,
    items: Vec<ItemRow>,
) -> Result<DefaultFormula, RepositoryError> {
    let internal = parse_internal(formula_row.internal_color_code)?;
    let customer = parse_customer(formula_row.customer_color_code)?;
    let base_kg = parse_kg_opt(formula_row.base_weight_kg)?;
    let ratio = parse_ratio_opt(formula_row.liquor_ratio)?;
    let created_at = parse_dt(&formula_row.created_at)?;
    let updated_at = parse_dt(&formula_row.updated_at)?;
    let items_dom: Result<Vec<_>, _> = items.into_iter().map(ItemRow::into_domain).collect();
    let items_dom = items_dom?;
    DefaultFormula::rehydrate(
        FormulaId::new(formula_row.id),
        internal,
        customer,
        formula_row.color_name,
        formula_row.description,
        base_kg,
        ratio,
        formula_row.notes,
        items_dom,
        created_at,
        updated_at,
    )
    .map_err(|e| corrupt("default_formula", e))
}

impl DefaultFormulaRepository for SqliteDefaultFormulaRepository {
    fn find_by_id(&self, id: FormulaId) -> Result<Option<DefaultFormula>, RepositoryError> {
        let result: Option<(FormulaRow, Vec<ItemRow>)> = self.db.with(|c| {
            let mut stmt = c.prepare(&format!(
                "SELECT {COLS} FROM default_formulas WHERE id = ?1"
            ))?;
            let mut rows = stmt.query(params![id.value()])?;
            match rows.next()? {
                None => Ok(None),
                Some(row) => {
                    let formula = FormulaRow::from_row(row)?;
                    let items = fetch_items(c, "default_formula_items", formula.id)?;
                    Ok(Some((formula, items)))
                }
            }
        })?;
        result
            .map(|(f, items)| assemble_default(f, items))
            .transpose()
    }

    fn find_by_internal_code(
        &self,
        code: &InternalColorCode,
    ) -> Result<Option<DefaultFormula>, RepositoryError> {
        let result: Option<(FormulaRow, Vec<ItemRow>)> = self.db.with(|c| {
            let mut stmt = c.prepare(&format!(
                "SELECT {COLS} FROM default_formulas WHERE internal_color_code = ?1"
            ))?;
            let mut rows = stmt.query(params![code.as_str()])?;
            match rows.next()? {
                None => Ok(None),
                Some(row) => {
                    let formula = FormulaRow::from_row(row)?;
                    let items = fetch_items(c, "default_formula_items", formula.id)?;
                    Ok(Some((formula, items)))
                }
            }
        })?;
        result
            .map(|(f, items)| assemble_default(f, items))
            .transpose()
    }

    fn find_by_customer_code(
        &self,
        customer_code: &str,
    ) -> Result<Vec<DefaultFormula>, RepositoryError> {
        let pairs: Vec<(FormulaRow, Vec<ItemRow>)> = self.db.with(|c| {
            let mut stmt = c.prepare(&format!(
                "SELECT {COLS} FROM default_formulas WHERE customer_color_code = ?1 ORDER BY id"
            ))?;
            let formulas: rusqlite::Result<Vec<FormulaRow>> = stmt
                .query_map(params![customer_code], FormulaRow::from_row)?
                .collect();
            let formulas = formulas?;
            let mut out = Vec::with_capacity(formulas.len());
            for f in formulas {
                let items = fetch_items(c, "default_formula_items", f.id)?;
                out.push((f, items));
            }
            Ok(out)
        })?;
        pairs
            .into_iter()
            .map(|(f, items)| assemble_default(f, items))
            .collect()
    }

    fn list(
        &self,
        query: DefaultFormulaQuery<'_>,
    ) -> Result<Vec<DefaultFormula>, RepositoryError> {
        let pairs: Vec<(FormulaRow, Vec<ItemRow>)> = self.db.with(|c| {
            let mut sql = format!("SELECT {COLS} FROM default_formulas");
            let mut bound: Vec<rusqlite::types::Value> = Vec::new();
            if let Some(kw) = query.keyword {
                let trimmed = kw.trim();
                if !trimmed.is_empty() {
                    sql.push_str(
                        " WHERE internal_color_code LIKE ?1
                              OR customer_color_code LIKE ?1
                              OR color_name LIKE ?1",
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
                let items = fetch_items(c, "default_formula_items", f.id)?;
                out.push((f, items));
            }
            Ok(out)
        })?;
        pairs
            .into_iter()
            .map(|(f, items)| assemble_default(f, items))
            .collect()
    }

    fn upsert(&self, formula: &DefaultFormula) -> Result<FormulaId, RepositoryError> {
        self.db.with_tx(|tx| {
            let id = match formula.id() {
                None => {
                    tx.execute(
                        "INSERT INTO default_formulas (internal_color_code, customer_color_code,
                            color_name, description, base_weight_kg, liquor_ratio, notes,
                            created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                        params![
                            <DefaultFormula as crate::domain::calculation::dye_calculator::CalculableFormula>::internal_color_code(formula).as_str(),
                            formula.customer_color_code().map(|c| c.as_str()),
                            formula.color_name(),
                            formula.description(),
                            formula.base_weight_kg().map(|k| k.value()),
                            <DefaultFormula as crate::domain::calculation::dye_calculator::CalculableFormula>::liquor_ratio(formula).map(|r| r.value()),
                            formula.notes(),
                            rfc3339(formula.created_at()),
                            rfc3339(formula.updated_at()),
                        ],
                    )?;
                    tx.last_insert_rowid()
                }
                Some(existing_id) => {
                    tx.execute(
                        "UPDATE default_formulas SET
                            internal_color_code = ?1,
                            customer_color_code = ?2,
                            color_name = ?3,
                            description = ?4,
                            base_weight_kg = ?5,
                            liquor_ratio = ?6,
                            notes = ?7,
                            updated_at = ?8
                         WHERE id = ?9",
                        params![
                            <DefaultFormula as crate::domain::calculation::dye_calculator::CalculableFormula>::internal_color_code(formula).as_str(),
                            formula.customer_color_code().map(|c| c.as_str()),
                            formula.color_name(),
                            formula.description(),
                            formula.base_weight_kg().map(|k| k.value()),
                            <DefaultFormula as crate::domain::calculation::dye_calculator::CalculableFormula>::liquor_ratio(formula).map(|r| r.value()),
                            formula.notes(),
                            rfc3339(formula.updated_at()),
                            existing_id.value(),
                        ],
                    )?;
                    existing_id.value()
                }
            };

            // 重写 items: 全删全插（单事务保证 items 一致性）。
            tx.execute(
                "DELETE FROM default_formula_items WHERE formula_id = ?1",
                params![id],
            )?;
            replace_items(
                tx,
                "default_formula_items",
                id,
                <DefaultFormula as crate::domain::calculation::dye_calculator::CalculableFormula>::items(formula),
            )?;

            Ok(FormulaId::new(id))
        })
    }

    fn delete(&self, id: FormulaId) -> Result<(), RepositoryError> {
        self.db.with(|c| {
            c.execute(
                "DELETE FROM default_formulas WHERE id = ?1",
                params![id.value()],
            )?;
            Ok(())
        })
    }
}

pub(crate) fn replace_items(
    tx: &Transaction<'_>,
    table: &str,
    formula_id: i64,
    items: &[FormulaItem],
) -> rusqlite::Result<()> {
    let mut stmt = tx.prepare(&format!(
        "INSERT INTO {table} (formula_id, dye_name, dye_code, percentage, unit, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
    ))?;
    for item in items {
        stmt.execute(params![
            formula_id,
            item.dye_name(),
            item.dye_code(),
            item.amount_value(),
            item.unit().as_db_str(),
            item.sort_order() as i64,
        ])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::formula::amounts::Kilograms;
    use crate::domain::formula::customer_color_code::CustomerColorCode;
    use crate::domain::formula::liquor_ratio::LiquorRatio;
    use crate::domain::formula::unit::Unit;
    use chrono::{TimeZone, Utc};

    fn db() -> Arc<SqliteConnection> {
        Arc::new(SqliteConnection::open_in_memory().unwrap())
    }

    fn t() -> chrono::DateTime<Utc> {
        Utc.timestamp_opt(0, 0).unwrap()
    }

    fn sample() -> DefaultFormula {
        DefaultFormula::new(
            InternalColorCode::new("N-2024").unwrap(),
            Some(CustomerColorCode::new("CN-001").unwrap()),
            Some("藏青".into()),
            Some("一种偏蓝的藏青色".into()),
            Some(Kilograms::new(50.0).unwrap()),
            Some(LiquorRatio::new(8.0).unwrap()),
            None,
            vec![
                FormulaItem::new("Reactive Blue 19", Some("RB19".into()), 2.0, Unit::PctOwf, 0)
                    .unwrap(),
                FormulaItem::new("Reactive Black 5", Some("RB5".into()), 1.5, Unit::PctOwf, 1)
                    .unwrap(),
                FormulaItem::new("Salt", None, 30.0, Unit::GramsPerL, 2).unwrap(),
            ],
            t(),
        )
        .unwrap()
    }

    #[test]
    fn insert_then_find_returns_full_formula() {
        let repo = SqliteDefaultFormulaRepository::new(db());
        let id = repo.upsert(&sample()).unwrap();
        let got = repo.find_by_id(id).unwrap().unwrap();
        let items = <DefaultFormula as crate::domain::calculation::dye_calculator::CalculableFormula>::items(&got);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].dye_name(), "Reactive Blue 19");
        assert_eq!(items[2].unit(), Unit::GramsPerL);
    }

    #[test]
    fn keyword_search_matches_internal_or_customer_or_name() {
        let repo = SqliteDefaultFormulaRepository::new(db());
        repo.upsert(&sample()).unwrap();
        for kw in ["N-2024", "CN-001", "藏青"] {
            let result = repo
                .list(DefaultFormulaQuery {
                    keyword: Some(kw),
                    limit: None,
                    offset: None,
                })
                .unwrap();
            assert_eq!(result.len(), 1, "keyword={kw}");
        }
    }

    #[test]
    fn upsert_updates_and_replaces_items() {
        let repo = SqliteDefaultFormulaRepository::new(db());
        let id = repo.upsert(&sample()).unwrap();
        let mut got = repo.find_by_id(id).unwrap().unwrap();
        got.replace_items(
            vec![
                FormulaItem::new("only", None, 0.5, Unit::PctOwf, 0).unwrap(),
            ],
            t(),
        )
        .unwrap();
        repo.upsert(&got).unwrap();
        let again = repo.find_by_id(id).unwrap().unwrap();
        let items = <DefaultFormula as crate::domain::calculation::dye_calculator::CalculableFormula>::items(&again);
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn duplicate_internal_code_is_conflict() {
        let repo = SqliteDefaultFormulaRepository::new(db());
        repo.upsert(&sample()).unwrap();
        let dup = sample();
        let err = repo.upsert(&dup).unwrap_err();
        assert!(matches!(err, RepositoryError::Conflict(_)));
    }
}
