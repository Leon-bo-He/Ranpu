//! Composition root：装配所有 adapter → service。
//!
//! 由 Tauri 命令 `boot_app` / `setup_first_run` 触发，将构造好的
//! `Services` 注入到 `AppState`。

use std::path::Path;
use std::sync::Arc;

use crate::application::audit::AuditService;
use crate::application::backup::BackupService;
use crate::application::calculation::CalculationService;
use crate::application::cart::CartService;
use crate::application::errors::{AppError, AppResult};
use crate::application::formula::FormulaService;
use crate::application::identity::IdentityService;
use crate::application::ports::password_hasher::PasswordHasher;
use crate::application::ports::user_repository::UserRepository;
use crate::application::workspace::WorkspaceService;
use crate::domain::calculation::dye_calculator::{DyeCalculator, StandardDyeCalculator};
use crate::infrastructure::clock_system::SystemClock;
use crate::infrastructure::crypto::{
    derive_db_key_hex, ensure_master_key, Argon2PasswordHasher, OsKeyStore, RanpuExporter,
};
use crate::infrastructure::export::{BatchSheetCsvExporter, PlainAuditCsvExporter};
use crate::infrastructure::persistence::seed;
use crate::infrastructure::persistence::{
    SqliteAuditRepository, SqliteAuditWriter, SqliteCartRepository, SqliteConnection,
    SqliteDbSnapshot, SqliteDefaultFormulaRepository, SqliteUserRepository,
    SqliteWorkspaceFormulaRepository, SqliteWorkspaceRepository,
};
use crate::infrastructure::session::InMemorySessionStore;
use crate::interfaces::tauri::state::{AppPaths, Services};

pub struct BootResult {
    pub services: Services,
    pub user_count: u64,
}

pub fn keystore_exists(paths: &AppPaths) -> bool {
    paths.keystore_path.exists()
}

pub fn boot(paths: &AppPaths, boot_passphrase: &str) -> AppResult<BootResult> {
    std::fs::create_dir_all(&paths.app_data_dir)
        .map_err(|e| AppError::Io(e.to_string()))?;
    std::fs::create_dir_all(&paths.tmp_dir).map_err(|e| AppError::Io(e.to_string()))?;

    let keystore = OsKeyStore::new(paths.keystore_path.clone());
    let master_key = ensure_master_key(&keystore).map_err(|e| AppError::Crypto(e.to_string()))?;
    let db_key_hex = derive_db_key_hex(&master_key, boot_passphrase);

    let db = open_db_or_wrong_passphrase(&paths.db_path, &db_key_hex)?;
    let db_arc = Arc::new(db);

    let user_repo = Arc::new(SqliteUserRepository::new(db_arc.clone()));
    let workspace_repo = Arc::new(SqliteWorkspaceRepository::new(db_arc.clone()));
    let default_repo = Arc::new(SqliteDefaultFormulaRepository::new(db_arc.clone()));
    let workspace_formula_repo =
        Arc::new(SqliteWorkspaceFormulaRepository::new(db_arc.clone()));
    let cart_repo = Arc::new(SqliteCartRepository::new(db_arc.clone()));
    let audit_repo_arc = Arc::new(SqliteAuditRepository::new(db_arc.clone()));
    let audit_writer = Arc::new(SqliteAuditWriter::new(db_arc.clone()));
    let snapshot = Arc::new(SqliteDbSnapshot::new(db_arc.clone(), paths.tmp_dir.clone()));

    let session_store = Arc::new(InMemorySessionStore::new());
    let clock = Arc::new(SystemClock::new());
    let hasher: Arc<dyn PasswordHasher> = Arc::new(Argon2PasswordHasher::new());
    let calculator: Arc<dyn DyeCalculator> = Arc::new(StandardDyeCalculator::new());
    let csv_audit = Arc::new(PlainAuditCsvExporter::new());
    let batch_sheet_exporter = Arc::new(BatchSheetCsvExporter::new());
    let yda = Arc::new(RanpuExporter::new());

    // 首次启动种子（仅当 workspace 空时插入）：
    let workspace_repo_dyn: Arc<dyn crate::application::ports::workspace_repository::WorkspaceRepository> =
        workspace_repo.clone();
    let default_repo_dyn: Arc<dyn crate::application::ports::default_formula_repository::DefaultFormulaRepository> =
        default_repo.clone();
    let workspace_formula_repo_dyn: Arc<dyn crate::application::ports::workspace_formula_repository::WorkspaceFormulaRepository> =
        workspace_formula_repo.clone();
    let _ = seed::run_if_empty(
        &workspace_repo_dyn,
        &default_repo_dyn,
        chrono::Utc::now(),
    )?;
    // 每次启动都跑一次系统镜像同步: 第一次创建 "通用" 工作区 + 后续自愈漂移.
    seed::ensure_system_mirror(
        &workspace_repo_dyn,
        &default_repo_dyn,
        &workspace_formula_repo_dyn,
        chrono::Utc::now(),
    )?;

    let user_count = user_repo.count()?;

    let identity = IdentityService::new(
        user_repo.clone(),
        hasher.clone(),
        audit_writer.clone(),
        clock.clone(),
        session_store.clone(),
    );
    let workspace = WorkspaceService::new(
        workspace_repo.clone(),
        audit_writer.clone(),
        clock.clone(),
        session_store.clone(),
    );
    let formula = FormulaService::new(
        default_repo.clone(),
        workspace_formula_repo.clone(),
        workspace_repo.clone(),
        audit_writer.clone(),
        clock.clone(),
        session_store.clone(),
        yda.clone(),
        yda.clone(),
    );
    let calculation = CalculationService::new(
        default_repo.clone(),
        workspace_formula_repo.clone(),
        calculator.clone(),
        audit_writer.clone(),
        clock.clone(),
        session_store.clone(),
    );
    let cart = CartService::new(
        cart_repo,
        default_repo,
        workspace_formula_repo,
        calculator,
        batch_sheet_exporter,
        audit_writer.clone(),
        clock.clone(),
        session_store.clone(),
    );
    let backup = BackupService::new(
        snapshot,
        yda.clone(),
        yda,
        audit_writer.clone(),
        clock.clone(),
        session_store.clone(),
    );
    let audit = AuditService::new(
        audit_repo_arc,
        audit_writer,
        csv_audit,
        Arc::new(RanpuExporter::new()),
        clock,
        session_store,
    );

    Ok(BootResult {
        services: Services {
            identity,
            workspace,
            formula,
            calculation,
            cart,
            backup,
            audit,
        },
        user_count,
    })
}

fn open_db_or_wrong_passphrase(
    db_path: &Path,
    db_key_hex: &str,
) -> AppResult<SqliteConnection> {
    match SqliteConnection::open(db_path, db_key_hex) {
        Ok(conn) => {
            // 显式做一次读取以确保 key 正确——SqliteConnection::open 会跑
            // schema (CREATE IF NOT EXISTS)，但完全空的 DB 也可能成功，因此
            // 再 SELECT 一下确保 SQLCipher 真的能解密。
            let probe: Result<i64, _> =
                conn.with(|c| c.query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get(0)));
            match probe {
                Ok(_) => Ok(conn),
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("not a database") || msg.contains("file is encrypted") {
                        Err(AppError::BootPassphraseIncorrect)
                    } else {
                        Err(AppError::Repository(e))
                    }
                }
            }
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not a database") || msg.contains("file is encrypted") {
                Err(AppError::BootPassphraseIncorrect)
            } else {
                Err(AppError::Repository(e))
            }
        }
    }
}
