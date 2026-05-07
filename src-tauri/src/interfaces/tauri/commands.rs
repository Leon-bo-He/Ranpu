//! Tauri 命令层。每个 #[tauri::command] 严格按 PROMPT 第 149 行 ≤ 30 行。

use std::str::FromStr;

use tauri::State;

use crate::application::audit::{AuditExportFormat, ExportAuditLogInput, ListAuditEventsInput};
use crate::application::backup::{ExportBackupInput, ImportBackupInput};
use crate::application::calculation::{CalculateDyeAmountsInput, SearchByCustomerCodeInput};
use crate::application::cart::{
    AddToCartInput, ExportCartInput, RemoveFromCartInput, UpdateCartItemKgInput,
};
use crate::application::formula::{
    BatchCopyDefaultInput, ExportDefaultFormulasInput, FormulaItemInput, FormulaUpsertInput,
    ImportDefaultFormulasInput, ListDefaultFormulasInput, ListWorkspaceFormulasInput,
};
use crate::application::identity::{
    AuthenticateUserInput, ChangeUserPasswordInput, CreateUserInput, UnlockOutcome,
    UnlockSessionInput,
};
use crate::application::ports::batch_sheet_exporter::BatchSheetFormat;
use crate::application::workspace::{CreateWorkspaceInput, RenameWorkspaceInput};
use crate::domain::audit::audit_event::Action;
use crate::domain::shared::id::{FormulaId, UserId, WorkspaceId};
use crate::interfaces::tauri::boot::{boot, keystore_exists};
use crate::interfaces::tauri::dto::*;
use crate::interfaces::tauri::error_mapping::{CmdResult, UiError};
use crate::interfaces::tauri::lock_guard::services_or_err;
use crate::interfaces::tauri::state::AppState;

// ---------- Boot / 首次启动 ----------

#[tauri::command]
pub fn cmd_boot_status(state: State<AppState>) -> BootStatusView {
    let booted = state.services().is_some();
    BootStatusView {
        keystore_exists: keystore_exists(&state.paths),
        db_initialized: booted,
        user_count: state
            .services()
            .map(|s| s.identity.list_users().map(|v| v.len() as u64).unwrap_or(0))
            .unwrap_or(0),
    }
}

#[tauri::command]
pub fn cmd_boot_app(state: State<AppState>, cmd: BootAppCmd) -> CmdResult<BootStatusView> {
    let result = boot(&state.paths, &cmd.boot_passphrase).map_err(UiError::from)?;
    let user_count = result.user_count;
    state.install(result.services);
    Ok(BootStatusView {
        keystore_exists: true,
        db_initialized: true,
        user_count,
    })
}

#[tauri::command]
pub fn cmd_setup_first_run(
    state: State<AppState>,
    cmd: CreateFirstAdminCmd,
) -> CmdResult<SessionView> {
    let result = boot(&state.paths, &cmd.boot_passphrase).map_err(UiError::from)?;
    state.install(result.services);
    let services = services_or_err(&state)?;
    services
        .identity
        .create_first_admin(CreateUserInput {
            username: cmd.username.clone(),
            password: cmd.password.clone(),
            role: crate::domain::identity::role::Role::Admin,
        })
        .map_err(UiError::from)?;
    let session = services
        .identity
        .authenticate_user(AuthenticateUserInput {
            username: cmd.username,
            password: cmd.password,
        })
        .map_err(UiError::from)?;
    Ok(SessionView::from(&session))
}

// ---------- Identity ----------

#[tauri::command]
pub fn cmd_login(state: State<AppState>, cmd: LoginCmd) -> CmdResult<SessionView> {
    let services = services_or_err(&state)?;
    let session = services
        .identity
        .authenticate_user(AuthenticateUserInput {
            username: cmd.username,
            password: cmd.password,
        })
        .map_err(UiError::from)?;
    Ok(SessionView::from(&session))
}

#[tauri::command]
pub fn cmd_logout(state: State<AppState>) -> CmdResult<()> {
    services_or_err(&state)?.identity.logout().map_err(UiError::from)
}

#[tauri::command]
pub fn cmd_lock_session(state: State<AppState>) -> CmdResult<()> {
    services_or_err(&state)?
        .identity
        .lock_session()
        .map_err(UiError::from)
}

#[tauri::command]
pub fn cmd_unlock_session(
    state: State<AppState>,
    cmd: UnlockSessionCmd,
) -> CmdResult<UnlockOutcomeView> {
    let outcome = services_or_err(&state)?
        .identity
        .unlock_session(UnlockSessionInput { password: cmd.password })
        .map_err(UiError::from)?;
    Ok(match outcome {
        UnlockOutcome::Unlocked => UnlockOutcomeView { kind: "unlocked".into(), remaining: None },
        UnlockOutcome::StillLocked { remaining } => {
            UnlockOutcomeView { kind: "still_locked".into(), remaining: Some(remaining) }
        }
        UnlockOutcome::ForceLoggedOut => {
            UnlockOutcomeView { kind: "force_logged_out".into(), remaining: None }
        }
    })
}

#[tauri::command]
pub fn cmd_change_password(state: State<AppState>, cmd: ChangePasswordCmd) -> CmdResult<()> {
    services_or_err(&state)?
        .identity
        .change_user_password(ChangeUserPasswordInput {
            old_password: cmd.old_password,
            new_password: cmd.new_password,
        })
        .map_err(UiError::from)
}

#[tauri::command]
pub fn cmd_create_user(state: State<AppState>, cmd: CreateUserCmd) -> CmdResult<i64> {
    let role = parse_role(&cmd.role).map_err(|m| UiError {
        code: "domain",
        message: m,
    })?;
    let id = services_or_err(&state)?
        .identity
        .create_user(CreateUserInput {
            username: cmd.username,
            password: cmd.password,
            role,
        })
        .map_err(UiError::from)?;
    Ok(id.value())
}

#[tauri::command]
pub fn cmd_deactivate_user(state: State<AppState>, user_id: i64) -> CmdResult<()> {
    services_or_err(&state)?
        .identity
        .deactivate_user(UserId::new(user_id))
        .map_err(UiError::from)
}

#[tauri::command]
pub fn cmd_list_users(state: State<AppState>) -> CmdResult<Vec<UserView>> {
    let users = services_or_err(&state)?.identity.list_users().map_err(UiError::from)?;
    Ok(users.iter().map(UserView::from).collect())
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
        color_name: cmd.color_name,
        description: cmd.description,
        base_weight_kg: cmd.base_weight_kg,
        liquor_ratio: cmd.liquor_ratio,
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
pub fn cmd_export_default_formulas(
    state: State<AppState>,
    cmd: ExportDefaultFormulasCmd,
) -> CmdResult<u32> {
    services_or_err(&state)?
        .formula
        .export_default_formulas(ExportDefaultFormulasInput {
            default_formula_ids: cmd.default_formula_ids.into_iter().map(FormulaId::new).collect(),
            passphrase: cmd.passphrase,
            out_path: cmd.out_path.into(),
        })
        .map_err(UiError::from)
}

#[tauri::command]
pub fn cmd_import_default_formulas(
    state: State<AppState>,
    cmd: ImportDefaultFormulasCmd,
) -> CmdResult<ImportFormulasSummaryView> {
    let summary = services_or_err(&state)?
        .formula
        .import_default_formulas(ImportDefaultFormulasInput {
            passphrase: cmd.passphrase,
            in_path: cmd.in_path.into(),
        })
        .map_err(UiError::from)?;
    Ok(ImportFormulasSummaryView::from(&summary))
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
            user_ids: cmd.user_ids.map(|v| v.into_iter().map(UserId::new).collect()),
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
            user_ids: cmd.user_ids.map(|v| v.into_iter().map(UserId::new).collect()),
            actions,
            format,
            passphrase: cmd.passphrase,
            out_path: cmd.out_path.into(),
        })
        .map_err(UiError::from)
}
