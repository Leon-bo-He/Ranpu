//! 染谱 Ranpu 后端入口（库 crate）。
//!
//! main.rs 仅做命令行入口；本文件负责拼装 Tauri Builder 与各模块。
//! Composition root（DB / 加密 / 服务）在 interfaces::tauri::boot 中。

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;

use std::path::PathBuf;

use tauri::Manager;

use crate::interfaces::tauri::commands::*;
use crate::interfaces::tauri::{AppPaths, AppState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 把 %APPDATA%/Ranpu 之类的目录解析出来，挂到 AppState。
            let app_data = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("Ranpu");
            std::fs::create_dir_all(&app_data).ok();
            let paths = AppPaths::new(app_data);
            app.manage(AppState::new(paths));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd_boot_status,
            cmd_boot_app,
            cmd_setup_first_run,
            cmd_login,
            cmd_logout,
            cmd_lock_session,
            cmd_unlock_session,
            cmd_change_password,
            cmd_create_user,
            cmd_deactivate_user,
            cmd_list_users,
            cmd_create_workspace,
            cmd_rename_workspace,
            cmd_list_workspaces,
            cmd_switch_workspace,
            cmd_delete_workspace,
            cmd_list_default_formulas,
            cmd_upsert_default_formula,
            cmd_delete_default_formula,
            cmd_list_workspace_formulas,
            cmd_upsert_workspace_formula,
            cmd_delete_workspace_formula,
            cmd_copy_default_to_active_workspace,
            cmd_calculate,
            cmd_add_to_cart,
            cmd_update_cart_kg,
            cmd_remove_from_cart,
            cmd_clear_cart,
            cmd_list_cart,
            cmd_export_cart,
            cmd_export_backup,
            cmd_import_backup,
            cmd_list_audit,
            cmd_export_audit,
        ])
        .run(tauri::generate_context!())
        .expect("启动染谱主窗口失败");
}
