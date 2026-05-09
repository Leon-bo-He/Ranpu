use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::{Mutex, RwLock};

use crate::application::audit::AuditService;
use crate::application::backup::BackupService;
use crate::application::calculation::CalculationService;
use crate::application::cart::CartService;
use crate::application::formula::FormulaService;
use crate::application::ports::SessionStore;
use crate::application::workspace::WorkspaceService;

/// 启动后才装好的所有应用服务（持久化连接已开 / SQLCipher 已 unlock）。
#[derive(Clone)]
pub struct Services {
    pub workspace: WorkspaceService,
    pub formula: FormulaService,
    pub calculation: CalculationService,
    pub cart: CartService,
    pub backup: BackupService,
    pub audit: AuditService,
    /// 共享会话存储 — 各服务也持有一份相同的 Arc, commands.rs 走这里直接
    /// 操作 lock / unlock 状态 (没有 IdentityService 包装).
    pub session_store: Arc<dyn SessionStore>,
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
///
/// `unlock_passphrase` 是 boot 成功时缓存的启动口令, 后续 cmd_unlock_session
/// 凭它在内存里比对 — 没有用户表 / hash 校验那一套, 重新输入就好.
/// 当然, 任何能拿到本进程内存的攻击者都能直接绕过, 我们防的是
/// 用户离开座位时同事顺手锁屏 / 解锁这种社工场景.
pub struct AppState {
    pub paths: AppPaths,
    pub services: RwLock<Option<Arc<Services>>>,
    pub unlock_passphrase: Mutex<Option<String>>,
}

impl AppState {
    pub fn new(paths: AppPaths) -> Self {
        Self {
            paths,
            services: RwLock::new(None),
            unlock_passphrase: Mutex::new(None),
        }
    }

    pub fn services(&self) -> Option<Arc<Services>> {
        self.services.read().clone()
    }

    pub fn install(&self, services: Services) {
        *self.services.write() = Some(Arc::new(services));
    }
}
