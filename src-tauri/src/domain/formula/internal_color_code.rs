use std::fmt;

use crate::domain::shared::errors::{DomainError, DomainResult};

/// 内部色号：1-32 字符、不含任何空白字符。
///
/// 唯一性由仓储/数据库索引保障：
/// - 在 default 配方库全局唯一
/// - 在每个 workspace 内按 (workspace_id, internal_color_code) 唯一
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InternalColorCode(String);

impl InternalColorCode {
    pub fn new(raw: impl Into<String>) -> DomainResult<Self> {
        let s = raw.into();
        let len = s.chars().count();
        if len == 0 {
            return Err(DomainError::InternalColorCodeEmpty);
        }
        if len > 32 {
            return Err(DomainError::InternalColorCodeTooLong { len });
        }
        if s.chars().any(char::is_whitespace) {
            return Err(DomainError::InternalColorCodeHasWhitespace);
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for InternalColorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_normal_code() {
        let v = InternalColorCode::new("N-2024").unwrap();
        assert_eq!(v.as_str(), "N-2024");
    }

    #[test]
    fn rejects_empty() {
        assert!(matches!(
            InternalColorCode::new(""),
            Err(DomainError::InternalColorCodeEmpty)
        ));
    }

    #[test]
    fn rejects_whitespace() {
        for s in ["N 2024", "N\t2024", "N\n2024"] {
            assert!(matches!(
                InternalColorCode::new(s),
                Err(DomainError::InternalColorCodeHasWhitespace)
            ));
        }
    }

    #[test]
    fn rejects_too_long() {
        let s = "X".repeat(33);
        assert!(matches!(
            InternalColorCode::new(s),
            Err(DomainError::InternalColorCodeTooLong { len: 33 })
        ));
    }

    #[test]
    fn accepts_boundary_lengths() {
        assert!(InternalColorCode::new("X").is_ok());
        assert!(InternalColorCode::new("X".repeat(32)).is_ok());
    }
}
