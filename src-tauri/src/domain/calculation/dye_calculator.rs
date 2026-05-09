//! 染料投料计算的领域服务。
//!
//! `DyeCalculator` 是 trait, 默认实现 `StandardDyeCalculator` 按两种公式算:
//!   pct_owf  → grams = target_kg * 1000 * pct / 100
//!   g_per_kg → grams = target_kg * amount
//!
//! 早期还有 g/L (需要 LiquorRatio), 1.0.7 起去掉.

use crate::domain::formula::amounts::{Grams, Kilograms};
use crate::domain::formula::formula_item::FormulaItem;
use crate::domain::formula::internal_color_code::InternalColorCode;
use crate::domain::formula::unit::Unit;
use crate::domain::shared::errors::{DomainError, DomainResult};
use crate::domain::shared::id::FormulaId;

/// 让 `DefaultFormula` / `WorkspaceFormula` 都能喂进 `DyeCalculator`。
pub trait CalculableFormula {
    fn internal_color_code(&self) -> &InternalColorCode;
    fn items(&self) -> &[FormulaItem];
}

/// 配方解析时的来源标记，用于 UI 角标显示「来自当前工作区」/「来自默认库」。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormulaSource {
    CurrentWorkspace,
    DefaultFallback,
}

impl FormulaSource {
    pub const fn display_label(self) -> &'static str {
        match self {
            FormulaSource::CurrentWorkspace => "来自当前工作区",
            FormulaSource::DefaultFallback => "来自默认库",
        }
    }
}

/// 单条染料的计算结果。
#[derive(Debug, Clone, PartialEq)]
pub struct CalculationLine {
    pub dye_name: String,
    pub dye_code: Option<String>,
    pub grams: Grams,
    pub unit_used: Unit,
}

/// 一次完整计算的结果。
#[derive(Debug, Clone, PartialEq)]
pub struct CalculationResult {
    pub source: FormulaSource,
    /// 解析到的具体配方 ID。计算器本身不知道（trait 没暴露），由
    /// application 层在 calculate 后回填，便于 UI 拿来「加入购物车」。
    pub formula_id: Option<FormulaId>,
    pub internal_color_code: InternalColorCode,
    pub target_kg: Kilograms,
    pub lines: Vec<CalculationLine>,
}

/// 染料投料计算器。
pub trait DyeCalculator: Send + Sync {
    fn calculate(
        &self,
        formula: &dyn CalculableFormula,
        target_kg: Kilograms,
        source: FormulaSource,
    ) -> DomainResult<CalculationResult>;
}

/// 标准实现。无状态、可并发共享，可放进 `Arc<dyn DyeCalculator>`。
#[derive(Debug, Default, Clone, Copy)]
pub struct StandardDyeCalculator;

impl StandardDyeCalculator {
    pub const fn new() -> Self {
        Self
    }
}

impl DyeCalculator for StandardDyeCalculator {
    fn calculate(
        &self,
        formula: &dyn CalculableFormula,
        target_kg: Kilograms,
        source: FormulaSource,
    ) -> DomainResult<CalculationResult> {
        if formula.items().is_empty() {
            return Err(DomainError::FormulaMustHaveAtLeastOneItem);
        }

        let mut lines = Vec::with_capacity(formula.items().len());
        for item in formula.items() {
            let grams_value = match item.unit() {
                // grams = target_kg * 1000 * pct / 100  ==  target_kg * 10 * pct
                Unit::PctOwf => target_kg.value() * 10.0 * item.amount_value(),
                // grams = target_kg * amount(g/kg)
                Unit::GramsPerKg => target_kg.value() * item.amount_value(),
            };
            lines.push(CalculationLine {
                dye_name: item.dye_name().to_owned(),
                dye_code: item.dye_code().map(str::to_owned),
                grams: Grams::new(grams_value)?,
                unit_used: item.unit(),
            });
        }

        Ok(CalculationResult {
            source,
            formula_id: None,
            internal_color_code: formula.internal_color_code().clone(),
            target_kg,
            lines,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 内联的轻量级 fixture，不依赖 DefaultFormula / WorkspaceFormula 聚合。
    struct FixtureFormula {
        code: InternalColorCode,
        items: Vec<FormulaItem>,
    }

    impl CalculableFormula for FixtureFormula {
        fn internal_color_code(&self) -> &InternalColorCode {
            &self.code
        }
        fn items(&self) -> &[FormulaItem] {
            &self.items
        }
    }

    fn fixture(items: Vec<FormulaItem>) -> FixtureFormula {
        FixtureFormula {
            code: InternalColorCode::new("X").unwrap(),
            items,
        }
    }

    fn item(amount: f64, unit: Unit) -> FormulaItem {
        FormulaItem::new("dye", Some("DC".into()), amount, unit, 0).unwrap()
    }

    #[test]
    fn pct_owf_ten_kg_at_two_pct_yields_two_hundred_grams() {
        // 10 kg * 1000 * 2 / 100 = 200 g
        let f = fixture(vec![item(2.0, Unit::PctOwf)]);
        let result = StandardDyeCalculator::new()
            .calculate(&f, Kilograms::new(10.0).unwrap(), FormulaSource::CurrentWorkspace)
            .unwrap();
        assert_eq!(result.lines.len(), 1);
        assert!((result.lines[0].grams.value() - 200.0).abs() < 1e-9);
        assert_eq!(result.source, FormulaSource::CurrentWorkspace);
    }

    #[test]
    fn pct_owf_boundary_smallest_kg_smallest_pct() {
        // 0.01 kg * 1000 * 0.001 / 100 = 0.0001 g
        let f = fixture(vec![item(0.001, Unit::PctOwf)]);
        let result = StandardDyeCalculator::new()
            .calculate(&f, Kilograms::new(0.01).unwrap(), FormulaSource::DefaultFallback)
            .unwrap();
        assert!((result.lines[0].grams.value() - 0.0001).abs() < 1e-12);
        assert_eq!(result.source, FormulaSource::DefaultFallback);
    }

    #[test]
    fn g_per_kg_directly_multiplies_target_kg() {
        // 50 kg * 3 g/kg = 150 g
        let f = fixture(vec![item(3.0, Unit::GramsPerKg)]);
        let r = StandardDyeCalculator::new()
            .calculate(&f, Kilograms::new(50.0).unwrap(), FormulaSource::CurrentWorkspace)
            .unwrap();
        assert!((r.lines[0].grams.value() - 150.0).abs() < 1e-9);
    }

    #[test]
    fn g_per_kg_with_decimal_amount() {
        // 25 kg * 0.5 g/kg = 12.5 g
        let f = fixture(vec![item(0.5, Unit::GramsPerKg)]);
        let r = StandardDyeCalculator::new()
            .calculate(&f, Kilograms::new(25.0).unwrap(), FormulaSource::CurrentWorkspace)
            .unwrap();
        assert!((r.lines[0].grams.value() - 12.5).abs() < 1e-9);
    }

    #[test]
    fn empty_items_is_rejected() {
        let f = fixture(vec![]);
        let err = StandardDyeCalculator::new()
            .calculate(&f, Kilograms::new(1.0).unwrap(), FormulaSource::CurrentWorkspace)
            .unwrap_err();
        assert!(matches!(err, DomainError::FormulaMustHaveAtLeastOneItem));
    }

    #[test]
    fn mixed_pct_and_g_per_kg_compute_independently() {
        // 10 kg, 1 个 pct_owf + 1 个 g/kg
        //   2% owf  -> 10 * 1000 * 2 / 100 = 200 g
        //   3 g/kg  -> 10 * 3 = 30 g
        let f = fixture(vec![
            item(2.0, Unit::PctOwf),
            item(3.0, Unit::GramsPerKg),
        ]);
        let r = StandardDyeCalculator::new()
            .calculate(&f, Kilograms::new(10.0).unwrap(), FormulaSource::CurrentWorkspace)
            .unwrap();
        assert_eq!(r.lines.len(), 2);
        assert!((r.lines[0].grams.value() - 200.0).abs() < 1e-9);
        assert!((r.lines[1].grams.value() - 30.0).abs() < 1e-9);
    }
}
