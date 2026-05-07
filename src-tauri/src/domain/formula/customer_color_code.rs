use std::fmt;

use crate::domain::shared::errors::{DomainError, DomainResult};

/// 客户色号：1-64 字符（可空，但若提供则不能为空字符串）。不强制唯一。
///
/// PROMPT 第 105 行：「CustomerColorCode（客户色号，可空，1–64 字符；不强制唯一）」。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomerColorCode(String);

impl CustomerColorCode {
    pub fn new(raw: impl Into<String>) -> DomainResult<Self> {
        let s = raw.into();
        let trimmed = s.trim();
        let len = trimmed.chars().count();
        if len == 0 {
            return Err(DomainError::CustomerColorCodeEmpty);
        }
        if len > 64 {
            return Err(DomainError::CustomerColorCodeTooLong { len });
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// 把 Option<String> 中的空白字符串归一为 None。
    pub fn maybe(raw: Option<String>) -> DomainResult<Option<Self>> {
        match raw {
            None => Ok(None),
            Some(s) if s.trim().is_empty() => Ok(None),
            Some(s) => Ok(Some(Self::new(s)?)),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for CustomerColorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_normal_value() {
        let c = CustomerColorCode::new("CUST-001").unwrap();
        assert_eq!(c.as_str(), "CUST-001");
    }

    #[test]
    fn trims_input() {
        let c = CustomerColorCode::new("  CUST  ").unwrap();
        assert_eq!(c.as_str(), "CUST");
    }

    #[test]
    fn rejects_empty_or_blank() {
        assert!(matches!(
            CustomerColorCode::new(""),
            Err(DomainError::CustomerColorCodeEmpty)
        ));
        assert!(matches!(
            CustomerColorCode::new("   "),
            Err(DomainError::CustomerColorCodeEmpty)
        ));
    }

    #[test]
    fn rejects_too_long() {
        let s = "x".repeat(65);
        assert!(matches!(
            CustomerColorCode::new(s),
            Err(DomainError::CustomerColorCodeTooLong { len: 65 })
        ));
    }

    #[test]
    fn maybe_handles_optional_input() {
        assert!(CustomerColorCode::maybe(None).unwrap().is_none());
        assert!(CustomerColorCode::maybe(Some("  ".into())).unwrap().is_none());
        let some = CustomerColorCode::maybe(Some("AB".into())).unwrap();
        assert_eq!(some.unwrap().as_str(), "AB");
    }
}
