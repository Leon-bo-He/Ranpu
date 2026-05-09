use std::fmt;
use std::str::FromStr;

use crate::domain::shared::errors::{DomainError, DomainResult};

/// 两种染料投料单位:
/// - PctOwf:    % owf (百分比, 相对纤维重量)
/// - GramsPerKg: g/kg (克每千克纤维)
///
/// 早期版本另有 GramsPerL (克每升染液), 依赖 LiquorRatio. 1.0.7 起去掉了浴比,
/// g_per_L 一并去除. 老 DB 里残留的 g_per_L 行由 connection.rs 迁移成 g_per_kg.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Unit {
    PctOwf,
    GramsPerKg,
}

impl Unit {
    /// 与数据库 CHECK 约束保持一致的字符串表示。
    pub const fn as_db_str(self) -> &'static str {
        match self {
            Unit::PctOwf => "pct_owf",
            Unit::GramsPerKg => "g_per_kg",
        }
    }

    /// UI 上展示的中文/通用标签。
    pub const fn display_label(self) -> &'static str {
        match self {
            Unit::PctOwf => "% (owf)",
            Unit::GramsPerKg => "g/kg",
        }
    }
}

impl FromStr for Unit {
    type Err = DomainError;

    fn from_str(s: &str) -> DomainResult<Self> {
        match s {
            "pct_owf" => Ok(Unit::PctOwf),
            "g_per_kg" => Ok(Unit::GramsPerKg),
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
        for u in [Unit::PctOwf, Unit::GramsPerKg] {
            assert_eq!(Unit::from_str(u.as_db_str()).unwrap(), u);
        }
    }

    #[test]
    fn rejects_unknown_string() {
        assert!(matches!(
            Unit::from_str("g_per_L"),
            Err(DomainError::UnknownUnit(s)) if s == "g_per_L"
        ));
    }

    #[test]
    fn display_uses_db_str() {
        assert_eq!(format!("{}", Unit::PctOwf), "pct_owf");
        assert_eq!(format!("{}", Unit::GramsPerKg), "g_per_kg");
    }
}
