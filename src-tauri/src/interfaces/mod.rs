//! Interfaces 层：与外部世界交接（Tauri 命令是目前唯一的接入面）。
//!
//! 只做 DTO 转换 + 调用 application + 权限/锁屏检查。
//! 每个 #[tauri::command] ≤ 30 行。
//! 子模块在后续 feat/interfaces-tauri 分支填充。

pub mod tauri {
    // 占位：commands、dto、error_mapping、state、lock_guard 在后续分支补齐
}
