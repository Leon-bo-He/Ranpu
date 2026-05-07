use std::fmt;
use std::str::FromStr;

use crate::domain::shared::errors::{DomainError, DomainResult};

/// 三种染料投料单位（PROMPT 第 109 行）：
/// - PctOwf:    % owf（百分比，相对纤维重量）
/// - GramsPerKg: g/kg（克每千克纤维）
/// - GramsPerL:  g/L  （克每升染液，需要 LiquorRatio 才能计算）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Unit {
    PctOwf,
    GramsPerKg,
    GramsPerL,
}

impl Unit {
    /// 与数据库 CHECK 约束保持一致的字符串表示。
    pub const fn as_db_str(self) -> &'static str {
        match self {
            Unit::PctOwf => "pct_owf",
            Unit::GramsPerKg => "g_per_kg",
            Unit::GramsPerL => "g_per_L",
        }
    }

    /// UI 上展示的中文/通用标签。
    pub const fn display_label(self) -> &'static str {
        match self {
            Unit::PctOwf => "% (owf)",
            Unit::GramsPerKg => "g/kg",
            Unit::GramsPerL => "g/L",
        }
    }

    pub const fn requires_liquor_ratio(self) -> bool {
        matches!(self, Unit::GramsPerL)
    }
}

impl FromStr for Unit {
    type Err = DomainError;

    fn from_str(s: &str) -> DomainResult<Self> {
        match s {
            "pct_owf" => Ok(Unit::PctOwf),
            "g_per_kg" => Ok(Unit::GramsPerKg),
            "g_per_L" => Ok(Unit::GramsPerL),
            other => Err(DomainError::UnknownUnit(other.to_owned())),
        }
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_db_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_db_str() {
        for u in [Unit::PctOwf, Unit::GramsPerKg, Unit::GramsPerL] {
            assert_eq!(Unit::from_str(u.as_db_str()).unwrap(), u);
        }
    }

    #[test]
    fn requires_liquor_ratio_only_for_g_per_l() {
        assert!(!Unit::PctOwf.requires_liquor_ratio());
        assert!(!Unit::GramsPerKg.requires_liquor_ratio());
        assert!(Unit::GramsPerL.requires_liquor_ratio());
    }

    #[test]
    fn rejects_unknown_string() {
        assert!(matches!(
            Unit::from_str("ml_per_kg"),
            Err(DomainError::UnknownUnit(s)) if s == "ml_per_kg"
        ));
    }

    #[test]
    fn display_uses_db_str() {
        assert_eq!(format!("{}", Unit::PctOwf), "pct_owf");
        assert_eq!(format!("{}", Unit::GramsPerL), "g_per_L");
    }
}
