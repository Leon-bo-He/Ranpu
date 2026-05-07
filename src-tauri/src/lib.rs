//! 染谱 Ranpu 后端入口（库 crate）。
//!
//! main.rs 仅做命令行入口；本文件负责拼装 Tauri Builder 与各模块。
//! 真正的 composition root（仓储/服务装配）会在 feat/interfaces-tauri 分支补齐。

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("启动染谱主窗口失败");
}
