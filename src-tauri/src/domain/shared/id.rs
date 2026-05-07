//! 强类型 ID 包装。
//!
//! 所有 ID 都是 i64（对应 SQLite 的 INTEGER PK），用 newtype 包一层避免
//! 跨上下文混用：UserId、WorkspaceId、FormulaId 等彼此不可隐式转换。

use std::fmt;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(i64);

        impl $name {
            #[inline]
            pub const fn new(raw: i64) -> Self {
                Self(raw)
            }

            #[inline]
            pub const fn value(self) -> i64 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<i64> for $name {
            fn from(raw: i64) -> Self {
                Self(raw)
            }
        }

        impl From<$name> for i64 {
            fn from(id: $name) -> Self {
                id.0
            }
        }
    };
}

id_type!(UserId);
id_type!(WorkspaceId);
id_type!(FormulaId);
id_type!(FormulaItemId);
id_type!(CartItemId);
id_type!(AuditEventId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_distinct_types() {
        let u = UserId::new(1);
        let w = WorkspaceId::new(1);
        // 编译期检查：UserId 与 WorkspaceId 不能混用，下行若解注释会编译失败：
        // let _: UserId = w;
        assert_eq!(u.value(), 1);
        assert_eq!(w.value(), 1);
    }

    #[test]
    fn id_round_trips_through_i64() {
        let id = FormulaId::from(42);
        let raw: i64 = id.into();
        assert_eq!(raw, 42);
    }
}
