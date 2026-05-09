//! Tauri 命令层。每个 #[tauri::command] 严格按 PROMPT 第 149 行 ≤ 30 行。
//!
//! 单用户解锁模型: 没有 cmd_login / cmd_logout / cmd_change_password /
//! cmd_create_user / cmd_*_user 这些 — 没有用户体系自然就没有这些命令.

use std::str::FromStr;

use tauri::State;

use crate::application::audit::{AuditExportFormat, ExportAuditLogInput, ListAuditEventsInput};
use crate::application::backup::{ExportBackupInput, ImportBackupInput};
use crate::application::calculation::{CalculateDyeAmountsInput, SearchByCustomerCodeInput};
use crate::application::cart::{
    AddToCartInput, ExportCartInput, PreviewBatchSheetInput, PreviewFormulaMetaInput,
    PreviewLayout, RemoveFromCartInput, UpdateCartItemKgInput,
};
use crate::application::errors::AppError;
use crate::application::formula::{
    BatchCopyDefaultInput, ExportArchiveInput, FormulaItemInput, FormulaUpsertInput,
    ImportArchiveInput, ListDefaultFormulasInput, ListWorkspaceFormulasInput,
    PreviewArchiveInput, WorkspaceImportPlan,
};
use crate::application::ports::batch_sheet_exporter::BatchSheetFormat;
use crate::application::workspace::{
    CreateWorkspaceInput, RenameWorkspaceInput, UpdateWorkspaceDescriptionInput,
};
use crate::domain::audit::audit_event::Action;
use crate::domain::session::Session;
use crate::domain::shared::id::{FormulaId, WorkspaceId};
use crate::interfaces::tauri::boot::{boot, keystore_exists};
use crate::interfaces::tauri::dto::*;
use crate::interfaces::tauri::error_mapping::{CmdResult, UiError};
use crate::interfaces::tauri::lock_guard::services_or_err;
use crate::interfaces::tauri::state::AppState;

// ---------- Boot / 首次启动 / 锁屏-解锁 ----------

#[tauri::command]
pub fn cmd_boot_status(state: State<AppState>) -> BootStatusView {
    let booted = state.services().is_some();
    BootStatusView {
        keystore_exists: keystore_exists(&state.paths),
        db_initialized: booted,
    }
}

#[tauri::command]
pub fn cmd_boot_app(state: State<AppState>, cmd: BootAppCmd) -> CmdResult<SessionView> {
    let result = boot(&state.paths, &cmd.boot_passphrase).map_err(UiError::from)?;
    install_services_and_session(&state, result.services, &cmd.boot_passphrase)
}

#[tauri::command]
pub fn cmd_setup_first_run(
    state: State<AppState>,
    cmd: BootAppCmd,
) -> CmdResult<SessionView> {
    // 单用户解锁模型: setup 与 boot 完全一样 — 第一次输入的口令就是
    // 后续的启动口令; SQLCipher 用它派生 DB key, keystore 在首次启动时
    // 自动写盘. 没有 admin / username / password 这一套.
    let result = boot(&state.paths, &cmd.boot_passphrase).map_err(UiError::from)?;
    install_services_and_session(&state, result.services, &cmd.boot_passphrase)
}

fn install_services_and_session(
    state: &State<AppState>,
    services: crate::interfaces::tauri::state::Services,
    boot_passphrase: &str,
) -> CmdResult<SessionView> {
    let session_store = services.session_store.clone();
    state.install(services);
    *state.unlock_passphrase.lock() = Some(boot_passphrase.to_owned());
    let session = Session::new(chrono::Utc::now());
    session_store.set(session.clone());
    Ok(SessionView::from(&session))
}

#[tauri::command]
pub fn cmd_lock_session(state: State<AppState>) -> CmdResult<()> {
    let services = services_or_err(&state)?;
    let now = chrono::Utc::now();
    let mutated = services
        .session_store
        .mutate(&mut |s| {
            s.lock();
            s.record_activity(now);
        });
    if !mutated {
        return Err(UiError::from(AppError::NotAuthenticated));
    }
    // 写一笔锁屏审计 (非致命: 失败也不影响锁屏).
    let event = crate::domain::audit::audit_event::AuditEvent::new(
        services
            .session_store
            .current()
            .and_then(|s| s.active_workspace_id()),
        Action::SessionLocked,
        None,
        None,
        now,
    );
    let _ = services
        .audit
        .write_event(&event); // 见下方 helper
    Ok(())
}

#[tauri::command]
pub fn cmd_unlock_session(
    state: State<AppState>,
    cmd: UnlockSessionCmd,
) -> CmdResult<SessionView> {
    let services = services_or_err(&state)?;
    let stored = state.unlock_passphrase.lock().clone();
    let expected = stored.ok_or_else(|| UiError::from(AppError::NotAuthenticated))?;
    if cmd.passphrase != expected {
        return Err(UiError::from(AppError::BootPassphraseIncorrect));
    }
    let now = chrono::Utc::now();
    let mutated = services
        .session_store
        .mutate(&mut |s| s.unlock(now));
    if !mutated {
        return Err(UiError::from(AppError::NotAuthenticated));
    }
    let event = crate::domain::audit::audit_event::AuditEvent::new(
        services
            .session_store
            .current()
            .and_then(|s| s.active_workspace_id()),
        Action::SessionUnlocked,
        None,
        None,
        now,
    );
    let _ = services.audit.write_event(&event);
    let session = services
        .session_store
        .current()
        .ok_or_else(|| UiError::from(AppError::NotAuthenticated))?;
    Ok(SessionView::from(&session))
}

// ---------- Workspace ----------

#[tauri::command]
pub fn cmd_create_workspace(
    state: State<AppState>,
    cmd: CreateWorkspaceCmd,
) -> CmdResult<i64> {
    let id = services_or_err(&state)?
        .workspace
        .create_workspace(CreateWorkspaceInput {
            name: cmd.name,
            description: cmd.description,
        })
        .map_err(UiError::from)?;
    Ok(id.value())
}

#[tauri::command]
pub fn cmd_rename_workspace(state: State<AppState>, cmd: RenameWorkspaceCmd) -> CmdResult<()> {
    services_or_err(&state)?
        .workspace
        .rename_workspace(RenameWorkspaceInput {
            workspace_id: WorkspaceId::new(cmd.workspace_id),
            new_name: cmd.new_name,
        })
        .map_err(UiError::from)
}

#[tauri::command]
pub fn cmd_update_workspace_description(
    state: State<AppState>,
    cmd: UpdateWorkspaceDescriptionCmd,
) -> CmdResult<()> {
    services_or_err(&state)?
        .workspace
        .update_workspace_description(UpdateWorkspaceDescriptionInput {
            workspace_id: WorkspaceId::new(cmd.workspace_id),
            description: cmd.description,
        })
        .map_err(UiError::from)
}

#[tauri::command]
pub fn cmd_list_workspaces(state: State<AppState>) -> CmdResult<Vec<WorkspaceView>> {
    let list = services_or_err(&state)?
        .workspace
        .list_workspaces()
        .map_err(UiError::from)?;
    Ok(list.iter().map(WorkspaceView::from).collect())
}

#[tauri::command]
pub fn cmd_switch_workspace(state: State<AppState>, cmd: SwitchWorkspaceCmd) -> CmdResult<()> {
    services_or_err(&state)?
        .workspace
        .switch_active_workspace(cmd.workspace_id.map(WorkspaceId::new))
        .map_err(UiError::from)
}

#[tauri::command]
pub fn cmd_delete_workspace(state: State<AppState>, workspace_id: i64) -> CmdResult<()> {
    services_or_err(&state)?
        .workspace
        .delete_workspace(WorkspaceId::new(workspace_id))
        .map_err(UiError::from)
}

// ---------- Formula ----------

fn cmd_to_upsert_input(cmd: UpsertFormulaCmd) -> FormulaUpsertInput {
    FormulaUpsertInput {
        id: cmd.id.map(FormulaId::new),
        internal_color_code: cmd.internal_color_code,
        customer_color_code: cmd.customer_color_code,
        color_family: cmd.color_family,
        notes: cmd.notes,
        items: cmd
            .items
            .into_iter()
            .map(|i| FormulaItemInput {
                dye_name: i.dye_name,
                dye_code: i.dye_code,
                amount: i.amount,
                unit: i.unit,
                sort_order: i.sort_order,
            })
            .collect(),
    }
}

#[tauri::command]
pub fn cmd_list_default_formulas(
    state: State<AppState>,
    cmd: ListFormulasCmd,
) -> CmdResult<Vec<FormulaView>> {
    let list = services_or_err(&state)?
        .formula
        .list_default_formulas(ListDefaultFormulasInput {
            keyword: cmd.keyword,
            limit: cmd.limit,
            offset: cmd.offset,
        })
        .map_err(UiError::from)?;
    Ok(list.iter().map(FormulaView::from).collect())
}

#[tauri::command]
pub fn cmd_upsert_default_formula(
    state: State<AppState>,
    cmd: UpsertFormulaCmd,
) -> CmdResult<i64> {
    let id = services_or_err(&state)?
        .formula
        .upsert_default_formula(cmd_to_upsert_input(cmd))
        .map_err(UiError::from)?;
    Ok(id.value())
}

#[tauri::command]
pub fn cmd_delete_default_formula(state: State<AppState>, id: i64) -> CmdResult<()> {
    services_or_err(&state)?
        .formula
        .delete_default_formula(FormulaId::new(id))
        .map_err(UiError::from)
}

#[tauri::command]
pub fn cmd_list_workspace_formulas(
    state: State<AppState>,
    cmd: ListFormulasCmd,
) -> CmdResult<Vec<FormulaView>> {
    let list = services_or_err(&state)?
        .formula
        .list_workspace_formulas(ListWorkspaceFormulasInput {
            keyword: cmd.keyword,
            limit: cmd.limit,
            offset: cmd.offset,
        })
        .map_err(UiError::from)?;
    Ok(list.iter().map(FormulaView::from).collect())
}

#[tauri::command]
pub fn cmd_upsert_workspace_formula(
    state: State<AppState>,
    cmd: UpsertFormulaCmd,
) -> CmdResult<i64> {
    let id = services_or_err(&state)?
        .formula
        .upsert_workspace_formula(cmd_to_upsert_input(cmd))
        .map_err(UiError::from)?;
    Ok(id.value())
}

#[tauri::command]
pub fn cmd_delete_workspace_formula(state: State<AppState>, id: i64) -> CmdResult<()> {
    services_or_err(&state)?
        .formula
        .delete_workspace_formula(FormulaId::new(id))
        .map_err(UiError::from)
}

#[tauri::command]
pub fn cmd_list_default_color_families(state: State<AppState>) -> CmdResult<Vec<String>> {
    services_or_err(&state)?
        .formula
        .list_default_color_families()
        .map_err(UiError::from)
}

#[tauri::command]
pub fn cmd_list_workspace_color_families(state: State<AppState>) -> CmdResult<Vec<String>> {
    services_or_err(&state)?
        .formula
        .list_workspace_color_families()
        .map_err(UiError::from)
}

#[tauri::command]
pub fn cmd_copy_default_to_active_workspace(
    state: State<AppState>,
    default_formula_id: i64,
) -> CmdResult<i64> {
    let new_id = services_or_err(&state)?
        .formula
        .copy_default_to_active_workspace(FormulaId::new(default_formula_id))
        .map_err(UiError::from)?;
    Ok(new_id.value())
}

#[tauri::command]
pub fn cmd_batch_copy_default_to_active_workspace(
    state: State<AppState>,
    cmd: BatchCopyDefaultCmd,
) -> CmdResult<BatchCopySummaryView> {
    let summary = services_or_err(&state)?
        .formula
        .batch_copy_default_to_active_workspace(BatchCopyDefaultInput {
            default_formula_ids: cmd.default_formula_ids.into_iter().map(FormulaId::new).collect(),
        })
        .map_err(UiError::from)?;
    Ok(BatchCopySummaryView::from(&summary))
}

#[tauri::command]
pub fn cmd_export_library_archive(
    state: State<AppState>,
    cmd: ExportLibraryArchiveCmd,
) -> CmdResult<ExportLibraryArchiveView> {
    let summary = services_or_err(&state)?
        .formula
        .export_library_archive(ExportArchiveInput {
            include_default: cmd.include_default,
            workspace_ids: cmd.workspace_ids.into_iter().map(WorkspaceId::new).collect(),
            passphrase: cmd.passphrase,
            out_path: cmd.out_path.into(),
        })
        .map_err(UiError::from)?;
    Ok(ExportLibraryArchiveView::from(&summary))
}

#[tauri::command]
pub fn cmd_preview_library_archive(
    state: State<AppState>,
    cmd: PreviewLibraryArchiveCmd,
) -> CmdResult<PreviewLibraryArchiveView> {
    let preview = services_or_err(&state)?
        .formula
        .preview_library_archive(PreviewArchiveInput {
            passphrase: cmd.passphrase,
            in_path: cmd.in_path.into(),
        })
        .map_err(UiError::from)?;
    Ok(PreviewLibraryArchiveView::from(&preview))
}

#[tauri::command]
pub fn cmd_import_library_archive(
    state: State<AppState>,
    cmd: ImportLibraryArchiveCmd,
) -> CmdResult<ImportLibraryArchiveView> {
    let mut plans = Vec::with_capacity(cmd.workspace_plans.len());
    for p in cmd.workspace_plans {
        let action = parse_workspace_import_action(&p.action)
            .map_err(|e| UiError::from(crate::application::errors::AppError::Internal(e)))?;
        plans.push(WorkspaceImportPlan {
            name: p.name,
            action,
        });
    }
    let summary = services_or_err(&state)?
        .formula
        .import_library_archive(ImportArchiveInput {
            passphrase: cmd.passphrase,
            in_path: cmd.in_path.into(),
            include_default: cmd.include_default,
            workspace_plans: plans,
        })
        .map_err(UiError::from)?;
    Ok(ImportLibraryArchiveView::from(&summary))
}

// ---------- Calculation ----------

#[tauri::command]
pub fn cmd_calculate(
    state: State<AppState>,
    cmd: CalculateCmd,
) -> CmdResult<CalculationResultView> {
    let result = services_or_err(&state)?
        .calculation
        .calculate_dye_amounts(CalculateDyeAmountsInput {
            internal_color_code: cmd.internal_color_code,
            target_kg: cmd.target_kg,
        })
        .map_err(UiError::from)?;
    Ok(CalculationResultView::from(&result))
}

#[tauri::command]
pub fn cmd_search_by_customer_code(
    state: State<AppState>,
    cmd: SearchByCustomerCodeCmd,
) -> CmdResult<Vec<CustomerCodeMatchView>> {
    let matches = services_or_err(&state)?
        .calculation
        .search_candidates_by_customer_code(SearchByCustomerCodeInput {
            customer_color_code: cmd.customer_color_code,
        })
        .map_err(UiError::from)?;
    Ok(matches.iter().map(CustomerCodeMatchView::from).collect())
}

// ---------- Cart ----------

#[tauri::command]
pub fn cmd_add_to_cart(state: State<AppState>, cmd: AddToCartCmd) -> CmdResult<()> {
    services_or_err(&state)?
        .cart
        .add_to_cart(AddToCartInput {
            source_kind: cmd.source_kind,
            source_formula_id: FormulaId::new(cmd.source_formula_id),
            target_kg: cmd.target_kg,
        })
        .map_err(UiError::from)
}

#[tauri::command]
pub fn cmd_update_cart_kg(state: State<AppState>, cmd: UpdateCartKgCmd) -> CmdResult<()> {
    services_or_err(&state)?
        .cart
        .update_cart_item_kg(UpdateCartItemKgInput {
            source_kind: cmd.source_kind,
            source_formula_id: FormulaId::new(cmd.source_formula_id),
            target_kg: cmd.target_kg,
        })
        .map_err(UiError::from)
}

#[tauri::command]
pub fn cmd_remove_from_cart(state: State<AppState>, cmd: RemoveFromCartCmd) -> CmdResult<bool> {
    services_or_err(&state)?
        .cart
        .remove_from_cart(RemoveFromCartInput {
            source_kind: cmd.source_kind,
            source_formula_id: FormulaId::new(cmd.source_formula_id),
        })
        .map_err(UiError::from)
}

#[tauri::command]
pub fn cmd_clear_cart(state: State<AppState>) -> CmdResult<()> {
    services_or_err(&state)?.cart.clear_cart().map_err(UiError::from)
}

#[tauri::command]
pub fn cmd_list_cart(state: State<AppState>) -> CmdResult<Vec<CartLineView>> {
    let lines = services_or_err(&state)?
        .cart
        .list_cart_with_calculations()
        .map_err(UiError::from)?;
    Ok(lines.iter().map(CartLineView::from_app).collect())
}

#[tauri::command]
pub fn cmd_export_cart(state: State<AppState>, cmd: ExportCartCmd) -> CmdResult<()> {
    let format = match cmd.format.as_str() {
        "csv" => BatchSheetFormat::Csv,
        "html" => BatchSheetFormat::Html,
        other => {
            return Err(UiError {
                code: "domain",
                message: format!("未知的批次单格式：{other}"),
            })
        }
    };
    services_or_err(&state)?
        .cart
        .export_cart_as_batch_sheet(ExportCartInput {
            format,
            out_path: cmd.out_path.into(),
        })
        .map_err(UiError::from)
}

#[tauri::command]
pub fn cmd_preview_cart_as_batch_sheet_html(
    state: State<AppState>,
    cmd: Option<PreviewCartCmd>,
) -> CmdResult<String> {
    let c = cmd.unwrap_or_default();
    let per_formula = c
        .per_formula
        .into_iter()
        .map(|m| PreviewFormulaMetaInput {
            vat_number: m.vat_number,
            yarn_count: m.yarn_count,
        })
        .collect();
    let layout = match c.layout.as_deref() {
        Some("grid") => PreviewLayout::Grid,
        _ => PreviewLayout::Standard,
    };
    services_or_err(&state)?
        .cart
        .preview_cart_as_batch_sheet_html(PreviewBatchSheetInput {
            customer: c.customer,
            per_formula,
            layout,
        })
        .map_err(UiError::from)
}

// ---------- Backup ----------

#[tauri::command]
pub fn cmd_export_backup(state: State<AppState>, cmd: ExportBackupCmd) -> CmdResult<()> {
    services_or_err(&state)?
        .backup
        .export_encrypted_backup(ExportBackupInput {
            passphrase: cmd.passphrase,
            out_path: cmd.out_path.into(),
        })
        .map_err(UiError::from)
}

#[tauri::command]
pub fn cmd_import_backup(state: State<AppState>, cmd: ImportBackupCmd) -> CmdResult<()> {
    services_or_err(&state)?
        .backup
        .import_encrypted_backup(ImportBackupInput {
            passphrase: cmd.passphrase,
            in_path: cmd.in_path.into(),
        })
        .map_err(UiError::from)
}

// ---------- Audit ----------

fn parse_actions(raw: Option<Vec<String>>) -> CmdResult<Option<Vec<Action>>> {
    raw.map(|v| {
        v.iter()
            .map(|s| Action::from_str(s))
            .collect::<Result<Vec<Action>, _>>()
            .map_err(|e| UiError {
                code: "domain",
                message: e.to_string(),
            })
    })
    .transpose()
}

#[tauri::command]
pub fn cmd_list_audit(
    state: State<AppState>,
    cmd: ListAuditCmd,
) -> CmdResult<Vec<AuditEventView>> {
    let actions = parse_actions(cmd.actions)?;
    let events = services_or_err(&state)?
        .audit
        .list_audit_events(ListAuditEventsInput {
            from: cmd.from,
            to: cmd.to,
            actions,
            limit: cmd.limit,
            offset: cmd.offset,
        })
        .map_err(UiError::from)?;
    Ok(events.iter().map(AuditEventView::from).collect())
}

#[tauri::command]
pub fn cmd_export_audit(state: State<AppState>, cmd: ExportAuditCmd) -> CmdResult<()> {
    let format = match cmd.format.as_str() {
        "encrypted" => AuditExportFormat::Encrypted,
        "csv" => AuditExportFormat::PlainCsv,
        other => {
            return Err(UiError {
                code: "domain",
                message: format!("未知的审计导出格式：{other}"),
            })
        }
    };
    let actions = parse_actions(cmd.actions)?;
    services_or_err(&state)?
        .audit
        .export_audit_log(ExportAuditLogInput {
            from: cmd.from,
            to: cmd.to,
            actions,
            format,
            passphrase: cmd.passphrase,
            out_path: cmd.out_path.into(),
        })
        .map_err(UiError::from)
}
