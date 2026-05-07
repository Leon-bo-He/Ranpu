use std::fmt;

use crate::domain::identity::errors::IdentityError;

/// 用户名值对象：1-64 字符、不含空白与控制字符。
///
/// 唯一性由仓储/数据库索引保障，不在值对象内校验。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Username(String);

impl Username {
    pub fn new(raw: impl Into<String>) -> Result<Self, IdentityError> {
        let s = raw.into();
        let len = s.chars().count();
        if len == 0 {
            return Err(IdentityError::UsernameEmpty);
        }
        if len > 64 {
            return Err(IdentityError::UsernameTooLong { len });
        }
        if s.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(IdentityError::UsernameHasInvalidChars);
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

impl fmt::Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// 密码哈希值对象。
///
/// 包装 argon2 PHC 字符串。本结构体不做格式校验——产生哈希的是
/// `application::ports::password_hasher::PasswordHasher` 的实现。
/// 关键作用是从类型层面隔离明文密码与哈希。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordHash(String);

impl PasswordHash {
    pub fn from_phc_string(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for PasswordHash {
    /// 不要直接打印整个哈希到日志（即便不是明文密码也属于敏感凭据）。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<password-hash:redacted>")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn username_accepts_normal_value() {
        let u = Username::new("alice").unwrap();
        assert_eq!(u.as_str(), "alice");
    }

    #[test]
    fn username_rejects_empty() {
        assert!(matches!(
            Username::new(""),
            Err(IdentityError::UsernameEmpty)
        ));
    }

    #[test]
    fn username_rejects_too_long() {
        let s = "a".repeat(65);
        assert!(matches!(
            Username::new(s),
            Err(IdentityError::UsernameTooLong { len: 65 })
        ));
    }

    #[test]
    fn username_rejects_whitespace_or_control() {
        assert!(matches!(
            Username::new("ali ce"),
            Err(IdentityError::UsernameHasInvalidChars)
        ));
        assert!(matches!(
            Username::new("ali\tce"),
            Err(IdentityError::UsernameHasInvalidChars)
        ));
        assert!(matches!(
            Username::new("ali\nce"),
            Err(IdentityError::UsernameHasInvalidChars)
        ));
    }

    #[test]
    fn password_hash_redacts_in_display() {
        let h = PasswordHash::from_phc_string("$argon2id$v=19$...$...");
        assert_eq!(format!("{h}"), "<password-hash:redacted>");
        assert!(h.as_str().starts_with("$argon2id$"));
    }
}
