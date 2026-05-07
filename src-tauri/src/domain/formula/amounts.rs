use std::fmt;

use crate::domain::formula::unit::Unit;
use crate::domain::shared::errors::{DomainError, DomainResult};

/// 百分比值（owf%）。
///
/// 必须是有限正数；上限不强制（允许 >100% 的高负载配方）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Percentage(f64);

impl Percentage {
    pub fn new(value: f64) -> DomainResult<Self> {
        if !value.is_finite() {
            return Err(DomainError::PercentageNotFinite);
        }
        if value <= 0.0 {
            return Err(DomainError::PercentageMustBePositive { actual: value });
        }
        Ok(Self(value))
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

impl fmt::Display for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}%", self.0)
    }
}

/// 染料计算结果克数（输出）。允许 0（理论极小占比），不允许负数 / NaN / Inf。
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Grams(f64);

impl Grams {
    pub fn new(value: f64) -> DomainResult<Self> {
        if !value.is_finite() {
            return Err(DomainError::GramsNotFinite);
        }
        if value < 0.0 {
            return Err(DomainError::GramsNegative { actual: value });
        }
        Ok(Self(value))
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

impl fmt::Display for Grams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} g", self.0)
    }
}

/// 计算输入：目标纤维 kg 数。范围 0.01 ~ 99999.99（PROMPT 第 300 行 UI 约束）。
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Kilograms(f64);

impl Kilograms {
    pub const MIN: f64 = 0.01;
    pub const MAX: f64 = 99_999.99;

    pub fn new(value: f64) -> DomainResult<Self> {
        if !value.is_finite() || !(Self::MIN..=Self::MAX).contains(&value) {
            return Err(DomainError::KilogramsOutOfRange { actual: value });
        }
        Ok(Self(value))
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

impl fmt::Display for Kilograms {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} kg", self.0)
    }
}

/// FormulaItem 中存放的「投料量 + 单位」。
///
/// value 的语义随 unit 变化（pct_owf 是百分比，g_per_kg / g_per_L 是克数）；
/// 这里只校验「正且有限」，业务上的合理范围由更上层（Formula 聚合）决定。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DyeAmount {
    value: f64,
    unit: Unit,
}

impl DyeAmount {
    pub fn new(value: f64, unit: Unit) -> DomainResult<Self> {
        if !value.is_finite() {
            return Err(DomainError::DyeAmountNotFinite);
        }
        if value <= 0.0 {
            return Err(DomainError::DyeAmountMustBePositive);
        }
        Ok(Self { value, unit })
    }

    pub fn value(self) -> f64 {
        self.value
    }

    pub fn unit(self) -> Unit {
        self.unit
    }
}

impl fmt::Display for DyeAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} {}", self.value, self.unit.display_label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentage_accepts_positive_finite() {
        assert!(Percentage::new(2.5).is_ok());
        assert!(Percentage::new(120.0).is_ok());
    }

    #[test]
    fn percentage_rejects_nonpositive() {
        assert!(matches!(
            Percentage::new(0.0),
            Err(DomainError::PercentageMustBePositive { .. })
        ));
        assert!(matches!(
            Percentage::new(-1.0),
            Err(DomainError::PercentageMustBePositive { .. })
        ));
    }

    #[test]
    fn grams_accepts_zero_and_positive() {
        assert!(Grams::new(0.0).is_ok());
        assert!(Grams::new(123.456).is_ok());
    }

    #[test]
    fn grams_rejects_negative_or_nonfinite() {
        assert!(matches!(
            Grams::new(-0.1),
            Err(DomainError::GramsNegative { .. })
        ));
        assert!(matches!(
            Grams::new(f64::INFINITY),
            Err(DomainError::GramsNotFinite)
        ));
    }

    #[test]
    fn kilograms_enforces_pratical_range() {
        assert!(Kilograms::new(0.01).is_ok());
        assert!(Kilograms::new(99_999.99).is_ok());
        assert!(matches!(
            Kilograms::new(0.0),
            Err(DomainError::KilogramsOutOfRange { .. })
        ));
        assert!(matches!(
            Kilograms::new(100_000.0),
            Err(DomainError::KilogramsOutOfRange { .. })
        ));
    }

    #[test]
    fn dye_amount_keeps_unit_with_value() {
        let a = DyeAmount::new(2.5, Unit::PctOwf).unwrap();
        assert_eq!(a.value(), 2.5);
        assert_eq!(a.unit(), Unit::PctOwf);
    }

    #[test]
    fn dye_amount_rejects_nonpositive() {
        assert!(matches!(
            DyeAmount::new(0.0, Unit::PctOwf),
            Err(DomainError::DyeAmountMustBePositive)
        ));
        assert!(matches!(
            DyeAmount::new(f64::NAN, Unit::PctOwf),
            Err(DomainError::DyeAmountNotFinite)
        ));
    }
}
