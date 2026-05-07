use chrono::{DateTime, Utc};

use crate::domain::calculation::dye_calculator::CalculableFormula;
use crate::domain::formula::amounts::Kilograms;
use crate::domain::formula::customer_color_code::CustomerColorCode;
use crate::domain::formula::formula_item::FormulaItem;
use crate::domain::formula::internal_color_code::InternalColorCode;
use crate::domain::formula::liquor_ratio::LiquorRatio;
use crate::domain::shared::errors::{DomainError, DomainResult};
use crate::domain::shared::id::{FormulaId, UserId};

/// 默认配方库的聚合根。
///
/// 不变量：
/// 1. items 至少 1 条（PROMPT 第 110 行）
/// 2. 任何 item 单位为 g_per_L 时，liquor_ratio 必须 Some（PROMPT 第 110 行）
/// 3. 内部色号在 default 库全局唯一 —— 由仓储/索引保障，不在聚合内校验
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFormula {
    id: Option<FormulaId>,
    internal_color_code: InternalColorCode,
    customer_color_code: Option<CustomerColorCode>,
    color_name: Option<String>,
    description: Option<String>,
    base_weight_kg: Option<Kilograms>,
    liquor_ratio: Option<LiquorRatio>,
    notes: Option<String>,
    items: Vec<FormulaItem>,
    created_by_user_id: Option<UserId>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[allow(clippy::too_many_arguments)]
impl DefaultFormula {
    pub fn new(
        internal_color_code: InternalColorCode,
        customer_color_code: Option<CustomerColorCode>,
        color_name: Option<String>,
        description: Option<String>,
        base_weight_kg: Option<Kilograms>,
        liquor_ratio: Option<LiquorRatio>,
        notes: Option<String>,
        items: Vec<FormulaItem>,
        created_by_user_id: Option<UserId>,
        created_at: DateTime<Utc>,
    ) -> DomainResult<Self> {
        let color_name = normalize_color_name(color_name)?;
        let description = normalize_description(description)?;
        let notes = normalize_notes(notes)?;
        check_invariants(&items, liquor_ratio)?;
        Ok(Self {
            id: None,
            internal_color_code,
            customer_color_code,
            color_name,
            description,
            base_weight_kg,
            liquor_ratio,
            notes,
            items,
            created_by_user_id,
            created_at,
            updated_at: created_at,
        })
    }

    pub fn rehydrate(
        id: FormulaId,
        internal_color_code: InternalColorCode,
        customer_color_code: Option<CustomerColorCode>,
        color_name: Option<String>,
        description: Option<String>,
        base_weight_kg: Option<Kilograms>,
        liquor_ratio: Option<LiquorRatio>,
        notes: Option<String>,
        items: Vec<FormulaItem>,
        created_by_user_id: Option<UserId>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> DomainResult<Self> {
        check_invariants(&items, liquor_ratio)?;
        Ok(Self {
            id: Some(id),
            internal_color_code,
            customer_color_code,
            color_name,
            description,
            base_weight_kg,
            liquor_ratio,
            notes,
            items,
            created_by_user_id,
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
    pub fn color_name(&self) -> Option<&str> {
        self.color_name.as_deref()
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn base_weight_kg(&self) -> Option<Kilograms> {
        self.base_weight_kg
    }
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
    pub fn created_by_user_id(&self) -> Option<UserId> {
        self.created_by_user_id
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
        check_invariants(&items, self.liquor_ratio)?;
        self.items = items;
        self.updated_at = now;
        Ok(())
    }

    pub fn set_liquor_ratio(
        &mut self,
        ratio: Option<LiquorRatio>,
        now: DateTime<Utc>,
    ) -> DomainResult<()> {
        check_invariants(&self.items, ratio)?;
        self.liquor_ratio = ratio;
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
}

impl CalculableFormula for DefaultFormula {
    fn internal_color_code(&self) -> &InternalColorCode {
        &self.internal_color_code
    }
    fn liquor_ratio(&self) -> Option<LiquorRatio> {
        self.liquor_ratio
    }
    fn items(&self) -> &[FormulaItem] {
        &self.items
    }
}

// ---------- 共享校验工具（同一聚合内可被多个 mutator 复用）----------

pub(crate) fn check_invariants(
    items: &[FormulaItem],
    liquor_ratio: Option<LiquorRatio>,
) -> DomainResult<()> {
    if items.is_empty() {
        return Err(DomainError::FormulaMustHaveAtLeastOneItem);
    }
    let needs_ratio = items.iter().any(|i| i.unit().requires_liquor_ratio());
    if needs_ratio && liquor_ratio.is_none() {
        return Err(DomainError::LiquorRatioRequired);
    }
    Ok(())
}

pub(crate) fn normalize_color_name(s: Option<String>) -> DomainResult<Option<String>> {
    normalize_optional_text(s, 64, |len| DomainError::ColorNameTooLong { len })
}

pub(crate) fn normalize_description(s: Option<String>) -> DomainResult<Option<String>> {
    normalize_optional_text(s, 1024, |len| DomainError::DescriptionTooLong { len })
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

    fn gpl_item() -> FormulaItem {
        FormulaItem::new("Salt", None, 5.0, Unit::GramsPerL, 1).unwrap()
    }

    #[test]
    fn new_with_pct_only_does_not_need_liquor_ratio() {
        let f = DefaultFormula::new(
            InternalColorCode::new("N-2024").unwrap(),
            None,
            Some("藏青".into()),
            None,
            None,
            None,
            None,
            vec![pct_item()],
            None,
            t(),
        )
        .unwrap();
        assert_eq!(f.color_name(), Some("藏青"));
        assert!(f.id().is_none());
    }

    #[test]
    fn new_rejects_empty_items() {
        let err = DefaultFormula::new(
            InternalColorCode::new("X").unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
            vec![],
            None,
            t(),
        )
        .unwrap_err();
        assert!(matches!(err, DomainError::FormulaMustHaveAtLeastOneItem));
    }

    #[test]
    fn new_rejects_g_per_l_without_liquor_ratio() {
        let err = DefaultFormula::new(
            InternalColorCode::new("X").unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
            vec![gpl_item()],
            None,
            t(),
        )
        .unwrap_err();
        assert!(matches!(err, DomainError::LiquorRatioRequired));
    }

    #[test]
    fn new_accepts_g_per_l_when_liquor_ratio_set() {
        let f = DefaultFormula::new(
            InternalColorCode::new("X").unwrap(),
            None,
            None,
            None,
            None,
            Some(LiquorRatio::new(8.0).unwrap()),
            None,
            vec![pct_item(), gpl_item()],
            None,
            t(),
        );
        assert!(f.is_ok());
    }

    #[test]
    fn replace_items_re_runs_invariants() {
        let mut f = DefaultFormula::new(
            InternalColorCode::new("X").unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
            vec![pct_item()],
            None,
            t(),
        )
        .unwrap();
        let err = f.replace_items(vec![gpl_item()], t()).unwrap_err();
        assert!(matches!(err, DomainError::LiquorRatioRequired));
        // items 不变
        assert_eq!(f.items().len(), 1);
        assert_eq!(f.items()[0].unit(), Unit::PctOwf);
    }

    #[test]
    fn set_liquor_ratio_to_none_with_g_per_l_items_is_rejected() {
        let mut f = DefaultFormula::new(
            InternalColorCode::new("X").unwrap(),
            None,
            None,
            None,
            None,
            Some(LiquorRatio::new(8.0).unwrap()),
            None,
            vec![gpl_item()],
            None,
            t(),
        )
        .unwrap();
        let err = f.set_liquor_ratio(None, t()).unwrap_err();
        assert!(matches!(err, DomainError::LiquorRatioRequired));
    }

    #[test]
    fn updated_at_advances_after_mutation() {
        let mut f = DefaultFormula::new(
            InternalColorCode::new("X").unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
            vec![pct_item()],
            None,
            t(),
        )
        .unwrap();
        assert_eq!(f.updated_at(), t());
        let later = t() + chrono::Duration::seconds(60);
        f.replace_items(vec![pct_item(), pct_item()], later).unwrap();
        assert_eq!(f.updated_at(), later);
    }
}
