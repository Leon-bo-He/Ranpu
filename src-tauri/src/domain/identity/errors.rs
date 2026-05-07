use chrono::{DateTime, Utc};
use thiserror::Error;

/// Identity 上下文领域错误。
///
/// 注意：登录失败的错误对外（UI）应统一为「账号或密码不对」防枚举，
/// 但领域层仍然区分具体原因，由 application/interfaces 层做统一映射。
#[derive(Debug, Error, Clone, PartialEq)]
pub enum IdentityError {
    #[error("用户名不能为空")]
    UsernameEmpty,
    #[error("用户名最长 64 个字符（当前 {len}）")]
    UsernameTooLong { len: usize },
    #[error("用户名不能包含空白或控制字符")]
    UsernameHasInvalidChars,

    #[error("密码不能为空")]
    PasswordEmpty,
    #[error("密码至少 {min} 位")]
    PasswordTooShort { min: usize },

    #[error("未知的角色：{0}")]
    UnknownRole(String),

    #[error("账号已停用")]
    AccountInactive,
    #[error("账号或密码不对")]
    InvalidCredentials,
    #[error("账号或密码不对，剩余 {remaining} 次机会")]
    InvalidCredentialsWithRemaining { remaining: u32 },
    #[error("已尝试 5 次都不对，账号已锁定 15 分钟，请稍后再来")]
    AccountJustLocked { until: DateTime<Utc> },
    #[error("账号已锁定，{until} 后再试")]
    AccountLocked { until: DateTime<Utc> },

    #[error("没有权限执行此操作")]
    PermissionDenied,
    #[error("用户名已被使用")]
    UsernameTaken,

    #[error("会话已锁定，请先解锁")]
    SessionLocked,
    #[error("尚未登录")]
    NotAuthenticated,
}
