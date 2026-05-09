//! 共享 Row → 领域对象 / 领域对象 → SQL 参数 的辅助工具。

use std::str::FromStr;

use chrono::{DateTime, Utc};
use rusqlite::Row;

use crate::application::ports::errors::RepositoryError;
use crate::domain::formula::customer_color_code::CustomerColorCode;
use crate::domain::formula::formula_item::FormulaItem;
use crate::domain::formula::internal_color_code::InternalColorCode;
use crate::domain::formula::unit::Unit;
use crate::domain::shared::id::FormulaItemId;

/// 把领域错误映射成 RepositoryError::Backend（数据损坏）。
pub fn corrupt(prefix: &str, err: impl std::fmt::Display) -> RepositoryError {
    RepositoryError::Backend(format!("{prefix}: {err}"))
}

pub fn parse_internal(s: String) -> Result<InternalColorCode, RepositoryError> {
    InternalColorCode::new(s).map_err(|e| corrupt("internal_color_code", e))
}

pub fn parse_customer(s: Option<String>) -> Result<Option<CustomerColorCode>, RepositoryError> {
    CustomerColorCode::maybe(s).map_err(|e| corrupt("customer_color_code", e))
}

pub fn parse_unit(s: &str) -> Result<Unit, RepositoryError> {
    Unit::from_str(s).map_err(|e| corrupt("unit", e))
}

pub fn parse_formula_item(row: &Row<'_>) -> Result<FormulaItem, RepositoryError> {
    let id: i64 = row.get(0).map_err(crate::infrastructure::persistence::sqlcipher::connection::map_err)?;
    let dye_name: String = row.get(1).map_err(crate::infrastructure::persistence::sqlcipher::connection::map_err)?;
    let dye_code: Option<String> = row.get(2).map_err(crate::infrastructure::persistence::sqlcipher::connection::map_err)?;
    let percentage: f64 = row.get(3).map_err(crate::infrastructure::persistence::sqlcipher::connection::map_err)?;
    let unit_str: String = row.get(4).map_err(crate::infrastructure::persistence::sqlcipher::connection::map_err)?;
    let sort_order: i64 = row.get(5).map_err(crate::infrastructure::persistence::sqlcipher::connection::map_err)?;
    let unit = parse_unit(&unit_str)?;
    FormulaItem::rehydrate(
        FormulaItemId::new(id),
        dye_name,
        dye_code,
        percentage,
        unit,
        sort_order.clamp(0, u16::MAX as i64) as u16,
    )
    .map_err(|e| corrupt("formula_item", e))
}

pub fn rfc3339(ts: DateTime<Utc>) -> String {
    ts.to_rfc3339()
}

pub fn parse_dt(s: &str) -> Result<DateTime<Utc>, RepositoryError> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| corrupt("datetime", e))
}

pub fn parse_dt_opt(s: Option<String>) -> Result<Option<DateTime<Utc>>, RepositoryError> {
    match s {
        None => Ok(None),
        Some(raw) => parse_dt(&raw).map(Some),
    }
}
