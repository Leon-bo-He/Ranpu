use std::fmt;

use crate::domain::shared::errors::{DomainError, DomainResult};

/// 浴比 1:N 中的 N。必须为正有限浮点。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquorRatio(f64);

impl LiquorRatio {
    pub fn new(value: f64) -> DomainResult<Self> {
        if !value.is_finite() {
            return Err(DomainError::LiquorRatioNotFinite);
        }
        if value <= 0.0 {
            return Err(DomainError::LiquorRatioMustBePositive { actual: value });
        }
        Ok(Self(value))
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

impl fmt::Display for LiquorRatio {
    /// 显示成「1:8」「1:12.50」这样的形式。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.fract().abs() < 1e-9 {
            write!(f, "1:{}", self.0 as i64)
        } else {
            write!(f, "1:{:.2}", self.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_positive() {
        assert_eq!(LiquorRatio::new(8.0).unwrap().value(), 8.0);
        assert_eq!(LiquorRatio::new(12.5).unwrap().value(), 12.5);
    }

    #[test]
    fn rejects_zero_and_negative() {
        assert!(matches!(
            LiquorRatio::new(0.0),
            Err(DomainError::LiquorRatioMustBePositive { .. })
        ));
        assert!(matches!(
            LiquorRatio::new(-1.0),
            Err(DomainError::LiquorRatioMustBePositive { .. })
        ));
    }

    #[test]
    fn rejects_nan_and_inf() {
        assert!(matches!(
            LiquorRatio::new(f64::NAN),
            Err(DomainError::LiquorRatioNotFinite)
        ));
        assert!(matches!(
            LiquorRatio::new(f64::INFINITY),
            Err(DomainError::LiquorRatioNotFinite)
        ));
    }

    #[test]
    fn display_is_human_readable() {
        assert_eq!(LiquorRatio::new(8.0).unwrap().to_string(), "1:8");
        assert_eq!(LiquorRatio::new(12.5).unwrap().to_string(), "1:12.50");
    }
}
