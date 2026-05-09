//! 应用层统一错误。
//!
//! 把 domain 错误、仓储错误、应用编排级错误（未解锁 / 未选 workspace）
//! 收拢到一个 enum，便于 interfaces/tauri 层统一映射成中文 UI 文案。
//!
//! 单用户解锁模型: 没有 IdentityError / PermissionDenied / AccountLocked /
//! InvalidCredentials 这些条目 — 没有用户 / 角色 / 登录概念.

use thiserror::Error;

use crate::application::ports::errors::RepositoryError;
use crate::domain::shared::errors::DomainError;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum AppError {
    #[error("{0}")]
    Domain(#[from] DomainError),

    #[error("{0}")]
    Repository(#[from] RepositoryError),

    #[error("尚未解锁")]
    NotAuthenticated,

    #[error("会话已锁定，请先解锁")]
    SessionLocked,

    #[error("请先选择工作区")]
    NoActiveWorkspace,

    #[error("启动口令不正确")]
    BootPassphraseIncorrect,

    #[error("文件读写出错：{0}")]
    Io(String),

    #[error("加密/解密出错：{0}")]
    Crypto(String),

    #[error("内部错误：{0}")]
    Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;
