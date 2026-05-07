//! Interfaces 层：与外部世界交接（Tauri 命令是目前唯一的接入面）。
//!
//! 只做 DTO 转换 + 调用 application + 权限/锁屏检查。
//! 每个 #[tauri::command] ≤ 30 行。

pub mod tauri;
