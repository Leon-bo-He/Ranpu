use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::{Mutex, RwLock};

use crate::application::audit::AuditService;
use crate::application::backup::BackupService;
use crate::application::calculation::CalculationService;
use crate::application::cart::CartService;
use crate::application::formula::FormulaService;
use crate::application::identity::IdentityService;
use crate::application::workspace::WorkspaceService;

/// 启动后才装好的所有应用服务（持久化连接已开 / SQLCipher 已 unlock）。
#[derive(Clone)]
pub struct Services {
    pub identity: IdentityService,
    pub workspace: WorkspaceService,
    pub formula: FormulaService,
    pub calculation: CalculationService,
    pub cart: CartService,
    pub backup: BackupService,
    pub audit: AuditService,
}

/// 启动前的运行时配置。
#[derive(Clone)]
pub struct AppPaths {
    /// %APPDATA%/Ranpu (Windows) 或者 ~/.local/share/ranpu (其它).
    pub app_data_dir: PathBuf,
    /// 主 DB 路径：{app_data_dir}/ranpu.db
    pub db_path: PathBuf,
    /// 主密钥库路径：{app_data_dir}/keystore.bin
    pub keystore_path: PathBuf,
    /// 临时目录（VACUUM INTO 用）：{app_data_dir}/tmp
    pub tmp_dir: PathBuf,
}

impl AppPaths {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let db_path = app_data_dir.join("ranpu.db");
        let keystore_path = app_data_dir.join("keystore.bin");
        let tmp_dir = app_data_dir.join("tmp");
        Self {
            app_data_dir,
            db_path,
            keystore_path,
            tmp_dir,
        }
    }
}

/// Tauri 注册的全局 State。RwLock 包 Option<Services>：boot 之前是 None。
pub struct AppState {
    pub paths: AppPaths,
    pub services: RwLock<Option<Arc<Services>>>,
    /// 一次性打印预览 HTML 缓冲: 主窗口 cmd_open_print_preview 写入,
    /// 新开的 print-preview 窗口 cmd_consume_print_preview 取出并清空.
    /// 目的是把可能很大的 HTML 字符串避开 URL / 命令行参数, 直接通过
    /// 进程内 state 传给子窗口.
    pub print_preview_html: Mutex<Option<String>>,
}

impl AppState {
    pub fn new(paths: AppPaths) -> Self {
        Self {
            paths,
            services: RwLock::new(None),
            print_preview_html: Mutex::new(None),
        }
    }

    pub fn services(&self) -> Option<Arc<Services>> {
        self.services.read().clone()
    }

    pub fn install(&self, services: Services) {
        *self.services.write() = Some(Arc::new(services));
    }
}
