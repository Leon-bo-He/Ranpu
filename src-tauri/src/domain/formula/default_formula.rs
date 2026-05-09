use chrono::{DateTime, Utc};

use crate::domain::calculation::dye_calculator::CalculableFormula;
use crate::domain::formula::customer_color_code::CustomerColorCode;
use crate::domain::formula::formula_item::FormulaItem;
use crate::domain::formula::internal_color_code::InternalColorCode;
use crate::domain::shared::errors::{DomainError, DomainResult};
use crate::domain::shared::id::FormulaId;

/// 默认配方库的聚合根。
///
/// 不变量：
/// 1. items 至少 1 条
/// 2. 内部色号在 default 库全局唯一 —— 由仓储/索引保障，不在聚合内校验
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFormula {
    id: Option<FormulaId>,
    internal_color_code: InternalColorCode,
    customer_color_code: Option<CustomerColorCode>,
    color_family: Option<String>,
    notes: Option<String>,
    items: Vec<FormulaItem>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[allow(clippy::too_many_arguments)]
impl DefaultFormula {
    pub fn new(
        internal_color_code: InternalColorCode,
        customer_color_code: Option<CustomerColorCode>,
        color_family: Option<String>,
        notes: Option<String>,
        items: Vec<FormulaItem>,
        created_at: DateTime<Utc>,
    ) -> DomainResult<Self> {
        let color_family = normalize_color_family(color_family)?;
        let notes = normalize_notes(notes)?;
        check_invariants(&items)?;
        Ok(Self {
            id: None,
            internal_color_code,
            customer_color_code,
            color_family,
            notes,
            items,
            created_at,
            updated_at: created_at,
        })
    }

    pub fn rehydrate(
        id: FormulaId,
        internal_color_code: InternalColorCode,
        customer_color_code: Option<CustomerColorCode>,
        color_family: Option<String>,
        notes: Option<String>,
        items: Vec<FormulaItem>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> DomainResult<Self> {
        check_invariants(&items)?;
        Ok(Self {
            id: Some(id),
            internal_color_code,
            customer_color_code,
            color_family,
            notes,
            items,
            created_at,
            updated_at,
        })
    }

    pub fn id(&self) -> Option<FormulaId> {
        self.id
    }
    pub fn customer_color_code(&self) -> Option<&CustomerColorCode> {
        self.customer_color_code.as_ref()
    }
    pub fn color_family(&self) -> Option<&str> {
        self.color_family.as_deref()
    }
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn assign_id(&mut self, id: FormulaId) {
        self.id = Some(id);
    }

    pub fn replace_items(
        &mut self,
        items: Vec<FormulaItem>,
        now: DateTime<Utc>,
    ) -> DomainResult<()> {
        check_invariants(&items)?;
        self.items = items;
        self.updated_at = now;
        Ok(())
    }

    pub fn rename_internal_code(
        &mut self,
        code: InternalColorCode,
        now: DateTime<Utc>,
    ) {
        self.internal_color_code = code;
        self.updated_at = now;
    }

    pub fn set_customer_color_code(
        &mut self,
        code: Option<CustomerColorCode>,
        now: DateTime<Utc>,
    ) {
        self.customer_color_code = code;
        self.updated_at = now;
    }

    pub fn set_color_family(
        &mut self,
        family: Option<String>,
        now: DateTime<Utc>,
    ) -> DomainResult<()> {
        self.color_family = normalize_color_family(family)?;
        self.updated_at = now;
        Ok(())
    }

    pub fn set_notes(&mut self, notes: Option<String>, now: DateTime<Utc>) -> DomainResult<()> {
        self.notes = normalize_notes(notes)?;
        self.updated_at = now;
        Ok(())
    }
}

impl CalculableFormula for DefaultFormula {
    fn internal_color_code(&self) -> &InternalColorCode {
        &self.internal_color_code
    }
    fn items(&self) -> &[FormulaItem] {
        &self.items
    }
}

// ---------- 共享校验工具（同一聚合内可被多个 mutator 复用）----------

pub(crate) fn check_invariants(items: &[FormulaItem]) -> DomainResult<()> {
    if items.is_empty() {
        return Err(DomainError::FormulaMustHaveAtLeastOneItem);
    }
    Ok(())
}

pub(crate) fn normalize_color_family(s: Option<String>) -> DomainResult<Option<String>> {
    normalize_optional_text(s, 32, |len| DomainError::ColorNameTooLong { len })
}

pub(crate) fn normalize_notes(s: Option<String>) -> DomainResult<Option<String>> {
    normalize_optional_text(s, 1024, |len| DomainError::DescriptionTooLong { len })
}

fn normalize_optional_text(
    s: Option<String>,
    max_len: usize,
    too_long: fn(usize) -> DomainError,
) -> DomainResult<Option<String>> {
    match s {
        None => Ok(None),
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            let len = trimmed.chars().count();
            if len > max_len {
                return Err(too_long(len));
            }
            Ok(Some(trimmed.to_owned()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::formula::unit::Unit;
    use chrono::TimeZone;

    fn t() -> DateTime<Utc> {
        Utc.timestamp_opt(0, 0).unwrap()
    }

    fn pct_item() -> FormulaItem {
        FormulaItem::new("Reactive Blue 19", Some("RB19".into()), 2.0, Unit::PctOwf, 0).unwrap()
    }

    #[test]
    fn new_with_pct_only_succeeds() {
        let f = DefaultFormula::new(
            InternalColorCode::new("N-2024").unwrap(),
            None,
            Some("蓝色系".into()),
            None,
            vec![pct_item()],
            t(),
        )
        .unwrap();
        assert_eq!(f.color_family(), Some("蓝色系"));
        assert!(f.id().is_none());
    }

    #[test]
    fn new_rejects_empty_items() {
        let err = DefaultFormula::new(
            InternalColorCode::new("X").unwrap(),
            None,
            None,
            None,
            vec![],
            t(),
        )
        .unwrap_err();
        assert!(matches!(err, DomainError::FormulaMustHaveAtLeastOneItem));
    }

    #[test]
    fn replace_items_keeps_invariants() {
        let mut f = DefaultFormula::new(
            InternalColorCode::new("X").unwrap(),
            None,
            None,
            None,
            vec![pct_item()],
            t(),
        )
        .unwrap();
        let err = f.replace_items(vec![], t()).unwrap_err();
        assert!(matches!(err, DomainError::FormulaMustHaveAtLeastOneItem));
        assert_eq!(f.items().len(), 1);
    }

    #[test]
    fn updated_at_advances_after_mutation() {
        let mut f = DefaultFormula::new(
            InternalColorCode::new("X").unwrap(),
            None,
            None,
            None,
            vec![pct_item()],
            t(),
        )
        .unwrap();
        assert_eq!(f.updated_at(), t());
        let later = t() + chrono::Duration::seconds(60);
        f.replace_items(vec![pct_item(), pct_item()], later).unwrap();
        assert_eq!(f.updated_at(), later);
    }

    #[test]
    fn set_color_family_normalizes_blank_to_none() {
        let mut f = DefaultFormula::new(
            InternalColorCode::new("X").unwrap(),
            None,
            Some("红色系".into()),
            None,
            vec![pct_item()],
            t(),
        )
        .unwrap();
        f.set_color_family(Some("   ".into()), t()).unwrap();
        assert_eq!(f.color_family(), None);
    }
}
