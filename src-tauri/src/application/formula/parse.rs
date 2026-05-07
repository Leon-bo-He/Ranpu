//! 把 DTO 输入转成领域聚合需要的值对象 / item 列表。

use std::str::FromStr;

use crate::application::errors::AppResult;
use crate::application::formula::service::{FormulaItemInput, FormulaUpsertInput};
use crate::domain::formula::amounts::Kilograms;
use crate::domain::formula::customer_color_code::CustomerColorCode;
use crate::domain::formula::formula_item::FormulaItem;
use crate::domain::formula::internal_color_code::InternalColorCode;
use crate::domain::formula::liquor_ratio::LiquorRatio;
use crate::domain::formula::unit::Unit;

pub fn parse_internal(input: &str) -> AppResult<InternalColorCode> {
    Ok(InternalColorCode::new(input.to_owned())?)
}

pub fn parse_customer(input: Option<String>) -> AppResult<Option<CustomerColorCode>> {
    Ok(CustomerColorCode::maybe(input)?)
}

pub fn parse_kg_opt(value: Option<f64>) -> AppResult<Option<Kilograms>> {
    match value {
        None => Ok(None),
        Some(v) => Ok(Some(Kilograms::new(v)?)),
    }
}

pub fn parse_ratio_opt(value: Option<f64>) -> AppResult<Option<LiquorRatio>> {
    match value {
        None => Ok(None),
        Some(v) => Ok(Some(LiquorRatio::new(v)?)),
    }
}

pub fn parse_items(items: &[FormulaItemInput]) -> AppResult<Vec<FormulaItem>> {
    let mut out = Vec::with_capacity(items.len());
    for it in items {
        let unit = Unit::from_str(&it.unit)?;
        out.push(FormulaItem::new(
            it.dye_name.clone(),
            it.dye_code.clone(),
            it.amount,
            unit,
            it.sort_order,
        )?);
    }
    Ok(out)
}

pub struct ParsedUpsert {
    pub id: Option<crate::domain::shared::id::FormulaId>,
    pub internal: InternalColorCode,
    pub customer: Option<CustomerColorCode>,
    pub color_name: Option<String>,
    pub description: Option<String>,
    pub base_kg: Option<Kilograms>,
    pub ratio: Option<LiquorRatio>,
    pub notes: Option<String>,
    pub items: Vec<FormulaItem>,
}

pub fn parse_upsert(input: FormulaUpsertInput) -> AppResult<ParsedUpsert> {
    Ok(ParsedUpsert {
        id: input.id,
        internal: parse_internal(&input.internal_color_code)?,
        customer: parse_customer(input.customer_color_code)?,
        color_name: input.color_name,
        description: input.description,
        base_kg: parse_kg_opt(input.base_weight_kg)?,
        ratio: parse_ratio_opt(input.liquor_ratio)?,
        notes: input.notes,
        items: parse_items(&input.items)?,
    })
}
