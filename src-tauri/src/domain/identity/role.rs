use std::fmt;
use std::str::FromStr;

use crate::domain::identity::errors::IdentityError;

/// 系统角色。
///
/// 仅两种：管理员（admin）与普通用户（user），都登录系统。
/// 权限差异通过本 enum 上的方法表达，避免 if/else 散落各处。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Admin,
    User,
}

impl Role {
    pub const fn as_db_str(self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::User => "user",
        }
    }

    pub const fn display_label(self) -> &'static str {
        match self {
            Role::Admin => "管理员",
            Role::User => "普通用户",
        }
    }

    /// 默认配方库的增删改（PROMPT 第 63 行）。
    pub const fn can_manage_default_formulas(self) -> bool {
        matches!(self, Role::Admin)
    }

    /// 工作区内配方的增删改 / 从默认库复制配方（PROMPT 第 219-222 行）。
    pub const fn can_manage_workspace_formulas(self) -> bool {
        matches!(self, Role::Admin)
    }

    /// 创建 / 重命名 / 删除工作区。
    pub const fn can_manage_workspaces(self) -> bool {
        matches!(self, Role::Admin)
    }

    /// 创建用户、停用用户、列出用户。
    pub const fn can_manage_users(self) -> bool {
        matches!(self, Role::Admin)
    }

    /// 查看 / 导出审计日志。
    pub const fn can_access_audit_log(self) -> bool {
        matches!(self, Role::Admin)
    }

    /// 任何登录用户都能：浏览配方、计算、用购物车、切换工作区。
    pub const fn can_read_formulas(self) -> bool {
        true
    }

    pub const fn can_calculate(self) -> bool {
        true
    }

    pub const fn can_use_cart(self) -> bool {
        true
    }

    pub const fn can_switch_workspace(self) -> bool {
        true
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_db_str())
    }
}

impl FromStr for Role {
    type Err = IdentityError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "admin" => Ok(Role::Admin),
            "user" => Ok(Role::User),
            other => Err(IdentityError::UnknownRole(other.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_has_full_access() {
        let r = Role::Admin;
        assert!(r.can_manage_default_formulas());
        assert!(r.can_manage_workspace_formulas());
        assert!(r.can_manage_workspaces());
        assert!(r.can_manage_users());
        assert!(r.can_access_audit_log());
        assert!(r.can_use_cart());
    }

    #[test]
    fn user_cannot_manage_anything() {
        let r = Role::User;
        assert!(!r.can_manage_default_formulas());
        assert!(!r.can_manage_workspace_formulas());
        assert!(!r.can_manage_workspaces());
        assert!(!r.can_manage_users());
        assert!(!r.can_access_audit_log());
    }

    #[test]
    fn user_can_read_and_calculate_and_cart() {
        let r = Role::User;
        assert!(r.can_read_formulas());
        assert!(r.can_calculate());
        assert!(r.can_use_cart());
        assert!(r.can_switch_workspace());
    }

    #[test]
    fn role_round_trips_through_db_str() {
        assert_eq!(Role::from_str("admin").unwrap(), Role::Admin);
        assert_eq!(Role::from_str("user").unwrap(), Role::User);
        assert_eq!(Role::Admin.as_db_str(), "admin");
        assert_eq!(Role::User.as_db_str(), "user");
    }

    #[test]
    fn unknown_role_string_is_rejected() {
        assert!(matches!(
            Role::from_str("super"),
            Err(IdentityError::UnknownRole(s)) if s == "super"
        ));
    }
}
