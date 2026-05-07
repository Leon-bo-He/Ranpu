//! 染料投料计算的领域服务。
//!
//! `DyeCalculator` 是 trait（也是 PROMPT (b) 节确认过的接口），
//! 默认实现 `StandardDyeCalculator` 按 PROMPT 第 119-122 行的三种公式计算。

use crate::domain::formula::amounts::{Grams, Kilograms};
use crate::domain::formula::formula_item::FormulaItem;
use crate::domain::formula::internal_color_code::InternalColorCode;
use crate::domain::formula::liquor_ratio::LiquorRatio;
use crate::domain::formula::unit::Unit;
use crate::domain::shared::errors::{DomainError, DomainResult};

/// 让 `DefaultFormula` / `WorkspaceFormula` 都能喂进 `DyeCalculator`。
pub trait CalculableFormula {
    fn internal_color_code(&self) -> &InternalColorCode;
    fn liquor_ratio(&self) -> Option<LiquorRatio>;
    fn items(&self) -> &[FormulaItem];
}

/// 配方解析时的来源标记，用于 UI 角标显示「来自当前工作区」/「来自默认库（fallback）」。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormulaSource {
    CurrentWorkspace,
    DefaultFallback,
}

impl FormulaSource {
    pub const fn display_label(self) -> &'static str {
        match self {
            FormulaSource::CurrentWorkspace => "来自当前工作区",
            FormulaSource::DefaultFallback => "来自默认库（fallback）",
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

        let needs_ratio = formula
            .items()
            .iter()
            .any(|i| i.unit().requires_liquor_ratio());
        let liquor_ratio = formula.liquor_ratio();
        if needs_ratio && liquor_ratio.is_none() {
            return Err(DomainError::LiquorRatioRequired);
        }

        let mut lines = Vec::with_capacity(formula.items().len());
        for item in formula.items() {
            let grams_value = match item.unit() {
                // grams = target_kg * 1000 * pct / 100  ==  target_kg * 10 * pct
                Unit::PctOwf => target_kg.value() * 10.0 * item.amount_value(),
                // grams = target_kg * amount(g/kg)
                Unit::GramsPerKg => target_kg.value() * item.amount_value(),
                // grams = target_kg * liquor_ratio * amount(g/L)
                Unit::GramsPerL => {
                    let ratio = liquor_ratio.expect("checked above").value();
                    target_kg.value() * ratio * item.amount_value()
                }
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
        ratio: Option<LiquorRatio>,
        items: Vec<FormulaItem>,
    }

    impl CalculableFormula for FixtureFormula {
        fn internal_color_code(&self) -> &InternalColorCode {
            &self.code
        }
        fn liquor_ratio(&self) -> Option<LiquorRatio> {
            self.ratio
        }
        fn items(&self) -> &[FormulaItem] {
            &self.items
        }
    }

    fn fixture(items: Vec<FormulaItem>, ratio: Option<LiquorRatio>) -> FixtureFormula {
        FixtureFormula {
            code: InternalColorCode::new("X").unwrap(),
            ratio,
            items,
        }
    }

    fn item(amount: f64, unit: Unit) -> FormulaItem {
        FormulaItem::new("dye", Some("DC".into()), amount, unit, 0).unwrap()
    }

    #[test]
    fn pct_owf_ten_kg_at_two_pct_yields_two_hundred_grams() {
        // 10 kg * 1000 * 2 / 100 = 200 g
        let f = fixture(vec![item(2.0, Unit::PctOwf)], None);
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
        let f = fixture(vec![item(0.001, Unit::PctOwf)], None);
        let result = StandardDyeCalculator::new()
            .calculate(&f, Kilograms::new(0.01).unwrap(), FormulaSource::DefaultFallback)
            .unwrap();
        assert!((result.lines[0].grams.value() - 0.0001).abs() < 1e-12);
        assert_eq!(result.source, FormulaSource::DefaultFallback);
    }

    #[test]
    fn g_per_kg_directly_multiplies_target_kg() {
        // 50 kg * 3 g/kg = 150 g
        let f = fixture(vec![item(3.0, Unit::GramsPerKg)], None);
        let r = StandardDyeCalculator::new()
            .calculate(&f, Kilograms::new(50.0).unwrap(), FormulaSource::CurrentWorkspace)
            .unwrap();
        assert!((r.lines[0].grams.value() - 150.0).abs() < 1e-9);
    }

    #[test]
    fn g_per_kg_with_decimal_amount() {
        // 25 kg * 0.5 g/kg = 12.5 g
        let f = fixture(vec![item(0.5, Unit::GramsPerKg)], None);
        let r = StandardDyeCalculator::new()
            .calculate(&f, Kilograms::new(25.0).unwrap(), FormulaSource::CurrentWorkspace)
            .unwrap();
        assert!((r.lines[0].grams.value() - 12.5).abs() < 1e-9);
    }

    #[test]
    fn g_per_l_uses_liquor_ratio() {
        // 10 kg * 8 (浴比 1:8) * 1.5 g/L = 120 g
        let ratio = LiquorRatio::new(8.0).unwrap();
        let f = fixture(vec![item(1.5, Unit::GramsPerL)], Some(ratio));
        let r = StandardDyeCalculator::new()
            .calculate(&f, Kilograms::new(10.0).unwrap(), FormulaSource::CurrentWorkspace)
            .unwrap();
        assert!((r.lines[0].grams.value() - 120.0).abs() < 1e-9);
    }

    #[test]
    fn g_per_l_without_ratio_is_rejected() {
        let f = fixture(vec![item(1.0, Unit::GramsPerL)], None);
        let err = StandardDyeCalculator::new()
            .calculate(&f, Kilograms::new(10.0).unwrap(), FormulaSource::CurrentWorkspace)
            .unwrap_err();
        assert!(matches!(err, DomainError::LiquorRatioRequired));
    }

    #[test]
    fn empty_items_is_rejected() {
        let f = fixture(vec![], None);
        let err = StandardDyeCalculator::new()
            .calculate(&f, Kilograms::new(1.0).unwrap(), FormulaSource::CurrentWorkspace)
            .unwrap_err();
        assert!(matches!(err, DomainError::FormulaMustHaveAtLeastOneItem));
    }

    #[test]
    fn mixed_units_compute_independently() {
        // 10 kg, 浴比 1:10, 含 1 个 pct_owf + 1 个 g/kg + 1 个 g/L
        // 2% owf  -> 10 * 1000 * 2 / 100 = 200 g
        // 3 g/kg  -> 10 * 3 = 30 g
        // 0.5 g/L (1:10) -> 10 * 10 * 0.5 = 50 g
        let f = fixture(
            vec![
                item(2.0, Unit::PctOwf),
                item(3.0, Unit::GramsPerKg),
                item(0.5, Unit::GramsPerL),
            ],
            Some(LiquorRatio::new(10.0).unwrap()),
        );
        let r = StandardDyeCalculator::new()
            .calculate(&f, Kilograms::new(10.0).unwrap(), FormulaSource::CurrentWorkspace)
            .unwrap();
        assert_eq!(r.lines.len(), 3);
        assert!((r.lines[0].grams.value() - 200.0).abs() < 1e-9);
        assert!((r.lines[1].grams.value() - 30.0).abs() < 1e-9);
        assert!((r.lines[2].grams.value() - 50.0).abs() < 1e-9);
    }
}
