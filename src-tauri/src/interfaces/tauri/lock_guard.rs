//! 命令级锁屏守卫：除特定命令外，会话被锁时所有命令立即拒绝。
//!
//! 业务用例自身已经在 ensure_active 守卫内做了 `is_locked()` 检查，
//! 这一层只是把「未启动」「无服务」也统一成 UI 友好提示。

use std::sync::Arc;

use crate::interfaces::tauri::error_mapping::UiError;
use crate::interfaces::tauri::state::{AppState, Services};

pub fn services_or_err(state: &AppState) -> Result<Arc<Services>, UiError> {
    state.services().ok_or(UiError {
        code: "not_booted",
        message: "应用尚未完成启动，请先输入启动口令".to_owned(),
    })
}
