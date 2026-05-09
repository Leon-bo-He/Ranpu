use crate::domain::formula::amounts::DyeAmount;
use crate::domain::formula::unit::Unit;
use crate::domain::shared::errors::{DomainError, DomainResult};
use crate::domain::shared::id::FormulaItemId;

/// Formula 子实体：一种染料的投料项。
///
/// `dye_name` 必填（≤64 字符），`dye_code` 选填（≤32 字符）。
/// `amount` 与 `unit` 一起决定克数：
///   pct_owf  → grams = target_kg * 1000 * amount / 100
///   g_per_kg → grams = target_kg * amount
/// 详见 `domain/calculation/dye_calculator.rs`。
#[derive(Debug, Clone, PartialEq)]
pub struct FormulaItem {
    id: Option<FormulaItemId>,
    dye_name: String,
    dye_code: Option<String>,
    amount: DyeAmount,
    sort_order: u16,
}

impl FormulaItem {
    pub fn new(
        dye_name: impl Into<String>,
        dye_code: Option<String>,
        amount_value: f64,
        unit: Unit,
        sort_order: u16,
    ) -> DomainResult<Self> {
        let dye_name = normalize_dye_name(dye_name.into())?;
        let dye_code = normalize_dye_code(dye_code)?;
        let amount = DyeAmount::new(amount_value, unit)?;
        Ok(Self {
            id: None,
            dye_name,
            dye_code,
            amount,
            sort_order,
        })
    }

    pub fn rehydrate(
        id: FormulaItemId,
        dye_name: String,
        dye_code: Option<String>,
        amount_value: f64,
        unit: Unit,
        sort_order: u16,
    ) -> DomainResult<Self> {
        let mut item = Self::new(dye_name, dye_code, amount_value, unit, sort_order)?;
        item.id = Some(id);
        Ok(item)
    }

    pub fn id(&self) -> Option<FormulaItemId> {
        self.id
    }

    pub fn dye_name(&self) -> &str {
        &self.dye_name
    }

    pub fn dye_code(&self) -> Option<&str> {
        self.dye_code.as_deref()
    }

    pub fn amount(&self) -> DyeAmount {
        self.amount
    }

    pub fn unit(&self) -> Unit {
        self.amount.unit()
    }

    pub fn amount_value(&self) -> f64 {
        self.amount.value()
    }

    pub fn sort_order(&self) -> u16 {
        self.sort_order
    }

    pub fn assign_id(&mut self, id: FormulaItemId) {
        self.id = Some(id);
    }
}

fn normalize_dye_name(s: String) -> DomainResult<String> {
    let trimmed = s.trim();
    let len = trimmed.chars().count();
    if len == 0 {
        return Err(DomainError::DyeNameEmpty);
    }
    if len > 64 {
        return Err(DomainError::DyeNameTooLong { len });
    }
    Ok(trimmed.to_owned())
}

fn normalize_dye_code(s: Option<String>) -> DomainResult<Option<String>> {
    match s {
        None => Ok(None),
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            let len = trimmed.chars().count();
            if len > 32 {
                return Err(DomainError::DyeCodeTooLong { len });
            }
            Ok(Some(trimmed.to_owned()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_item_holds_all_fields() {
        let item = FormulaItem::new(
            "C.I. Reactive Blue 19",
            Some("RB19".into()),
            2.5,
            Unit::PctOwf,
            0,
        )
        .unwrap();
        assert_eq!(item.dye_name(), "C.I. Reactive Blue 19");
        assert_eq!(item.dye_code(), Some("RB19"));
        assert_eq!(item.amount_value(), 2.5);
        assert_eq!(item.unit(), Unit::PctOwf);
        assert_eq!(item.sort_order(), 0);
        assert!(item.id().is_none());
    }

    #[test]
    fn empty_dye_name_is_rejected() {
        assert!(matches!(
            FormulaItem::new("  ", None, 1.0, Unit::PctOwf, 0),
            Err(DomainError::DyeNameEmpty)
        ));
    }

    #[test]
    fn long_dye_name_is_rejected() {
        let s = "X".repeat(65);
        assert!(matches!(
            FormulaItem::new(s, None, 1.0, Unit::PctOwf, 0),
            Err(DomainError::DyeNameTooLong { len: 65 })
        ));
    }

    #[test]
    fn blank_dye_code_normalizes_to_none() {
        let item = FormulaItem::new("X", Some("  ".into()), 1.0, Unit::PctOwf, 0).unwrap();
        assert_eq!(item.dye_code(), None);
    }

    #[test]
    fn long_dye_code_is_rejected() {
        let code = "C".repeat(33);
        assert!(matches!(
            FormulaItem::new("X", Some(code), 1.0, Unit::PctOwf, 0),
            Err(DomainError::DyeCodeTooLong { len: 33 })
        ));
    }

    #[test]
    fn nonpositive_amount_is_rejected() {
        assert!(matches!(
            FormulaItem::new("X", None, 0.0, Unit::PctOwf, 0),
            Err(DomainError::DyeAmountMustBePositive)
        ));
    }
}
