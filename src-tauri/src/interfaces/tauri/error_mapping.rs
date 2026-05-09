use serde::Serialize;

use crate::application::errors::AppError;

/// 给前端的错误结构。`code` 用于程序判断（i18n 已经在 message 上做完，
/// 但 code 可让前端做特殊跳转，比如锁屏 / 未解锁 → 回解锁页）.
#[derive(Debug, Serialize)]
pub struct UiError {
    pub code: &'static str,
    pub message: String,
}

impl From<AppError> for UiError {
    fn from(err: AppError) -> Self {
        let code = match &err {
            AppError::Domain(_) => "domain",
            AppError::Repository(_) => "repository",
            AppError::NotAuthenticated => "not_authenticated",
            AppError::SessionLocked => "session_locked",
            AppError::NoActiveWorkspace => "no_active_workspace",
            AppError::BootPassphraseIncorrect => "boot_passphrase_incorrect",
            AppError::Io(_) => "io",
            AppError::Crypto(_) => "crypto",
            AppError::Internal(_) => "internal",
        };
        Self {
            code,
            message: err.to_string(),
        }
    }
}

pub type CmdResult<T> = Result<T, UiError>;
