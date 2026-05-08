//! 染谱 Ranpu 后端入口（库 crate）。
//!
//! main.rs 仅做命令行入口；本文件负责拼装 Tauri Builder 与各模块。
//! Composition root（DB / 加密 / 服务）在 interfaces::tauri::boot 中。

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;

use std::borrow::Cow;
use std::path::PathBuf;

use tauri::Manager;

use crate::interfaces::tauri::commands::*;
use crate::interfaces::tauri::{AppPaths, AppState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // 单实例守门: 第二个进程启动会被插件直接退出, 并把 (argv, cwd) 通过
        // IPC 推回老实例; 我们在回调里把已有窗口取消最小化 + 抢回焦点.
        // 必须最先注册 (Tauri 官方文档): 越早拦截第二实例越好.
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            use tauri::Manager;
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        // 自定义 URI 协议: 打印预览窗口加载 ranpu-print://localhost/preview,
        // handler 直接从 AppState 取出 stash 的 HTML 返回, 不走 SPA / Vite /
        // React. 自包含 HTML (内联工具栏 + window.print() / window.close())
        // 让新窗口完全不依赖 Tauri JS API.
        .register_uri_scheme_protocol("ranpu-print", |ctx, _req| {
            let app = ctx.app_handle();
            let state = app.state::<AppState>();
            let html = state
                .print_preview_html
                .lock()
                .clone()
                .unwrap_or_else(|| String::from("<!DOCTYPE html><meta charset=utf-8><body>(no content)</body>"));
            let body: Cow<'static, [u8]> = Cow::Owned(html.into_bytes());
            tauri::http::Response::builder()
                .status(200)
                .header("Content-Type", "text/html; charset=utf-8")
                .body(body)
                .unwrap()
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // 数据放在 %APPDATA%\Ranpu (Windows) 或 ~/.config/Ranpu (Linux)
            // 等位置, 用 OS 的 base data 目录而非 Tauri 默认的
            // <data_dir>/<bundle-identifier>，避免出现 %APPDATA%\com.ranpu.app\Ranpu
            // 这种丑路径，也方便用户和 README 直接对位删除。
            let app_data = app
                .path()
                .data_dir()
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
            cmd_activate_user,
            cmd_list_users,
            cmd_create_workspace,
            cmd_rename_workspace,
            cmd_update_workspace_description,
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
            cmd_batch_copy_default_to_active_workspace,
            cmd_export_library_archive,
            cmd_preview_library_archive,
            cmd_import_library_archive,
            cmd_calculate,
            cmd_search_by_customer_code,
            cmd_add_to_cart,
            cmd_update_cart_kg,
            cmd_remove_from_cart,
            cmd_clear_cart,
            cmd_list_cart,
            cmd_export_cart,
            cmd_open_print_preview,
            cmd_export_backup,
            cmd_import_backup,
            cmd_list_audit,
            cmd_export_audit,
        ])
        .run(tauri::generate_context!())
        .expect("启动染谱主窗口失败");
}
