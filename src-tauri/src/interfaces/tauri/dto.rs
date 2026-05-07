//! 与前端交互的所有 DTO（serde 序列化）。
//!
//! 命名约定：Cmd 是命令入参，View 是返回。所有时间字段输出 RFC3339 字符串，
//! 前端按 PROMPT 第 299 行 `YYYY-MM-DD HH:mm` 自行格式化。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::audit::audit_event::AuditEvent;
use crate::domain::calculation::dye_calculator::{
    CalculableFormula, CalculationLine, CalculationResult, FormulaSource,
};
use crate::domain::cart::cart_item::CartItem;
use crate::domain::formula::default_formula::DefaultFormula;
use crate::domain::formula::workspace_formula::WorkspaceFormula;
use crate::domain::identity::role::Role;
use crate::domain::identity::session::Session;
use crate::domain::identity::user::User;
use crate::domain::workspace::workspace::Workspace;

// ---------- Identity ----------

#[derive(Debug, Deserialize)]
pub struct LoginCmd {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UnlockSessionCmd {
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordCmd {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserCmd {
    pub username: String,
    pub password: String,
    pub role: String, // "admin" | "user"
}

#[derive(Debug, Deserialize)]
pub struct CreateFirstAdminCmd {
    pub boot_passphrase: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct BootAppCmd {
    pub boot_passphrase: String,
}

#[derive(Debug, Serialize)]
pub struct SessionView {
    pub user_id: i64,
    pub username: String,
    pub role: String,
    pub active_workspace_id: Option<i64>,
    pub locked: bool,
    pub last_activity_at: DateTime<Utc>,
}

impl From<&Session> for SessionView {
    fn from(s: &Session) -> Self {
        Self {
            user_id: s.user_id().value(),
            username: s.username().as_str().to_owned(),
            role: s.role().as_db_str().to_owned(),
            active_workspace_id: s.active_workspace_id().map(|i| i.value()),
            locked: s.is_locked(),
            last_activity_at: s.last_activity_at(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UnlockOutcomeView {
    pub kind: String,
    pub remaining: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct UserView {
    pub id: i64,
    pub username: String,
    pub role: String,
    pub is_active: bool,
    pub failed_attempts: u32,
    pub locked_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

impl From<&User> for UserView {
    fn from(u: &User) -> Self {
        Self {
            id: u.id().expect("persisted").value(),
            username: u.username().as_str().to_owned(),
            role: u.role().as_db_str().to_owned(),
            is_active: u.is_active(),
            failed_attempts: u.failed_attempts(),
            locked_until: u.locked_until(),
            created_at: u.created_at(),
            last_login: u.last_login(),
        }
    }
}

// ---------- Workspace ----------

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceCmd {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RenameWorkspaceCmd {
    pub workspace_id: i64,
    pub new_name: String,
}

#[derive(Debug, Deserialize)]
pub struct SwitchWorkspaceCmd {
    pub workspace_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceView {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<&Workspace> for WorkspaceView {
    fn from(w: &Workspace) -> Self {
        Self {
            id: w.id().expect("persisted").value(),
            name: w.name().as_str().to_owned(),
            description: w.description().map(str::to_owned),
            created_at: w.created_at(),
        }
    }
}

// ---------- Formula ----------

#[derive(Debug, Deserialize)]
pub struct ListFormulasCmd {
    pub keyword: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertFormulaCmd {
    pub id: Option<i64>,
    pub internal_color_code: String,
    pub customer_color_code: Option<String>,
    pub color_name: Option<String>,
    pub description: Option<String>,
    pub base_weight_kg: Option<f64>,
    pub liquor_ratio: Option<f64>,
    pub notes: Option<String>,
    pub items: Vec<FormulaItemDto>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FormulaItemDto {
    pub dye_name: String,
    pub dye_code: Option<String>,
    pub amount: f64,
    pub unit: String,
    pub sort_order: u16,
}

#[derive(Debug, Serialize)]
pub struct FormulaView {
    pub id: i64,
    pub internal_color_code: String,
    pub customer_color_code: Option<String>,
    pub color_name: Option<String>,
    pub description: Option<String>,
    pub base_weight_kg: Option<f64>,
    pub liquor_ratio: Option<f64>,
    pub notes: Option<String>,
    pub items: Vec<FormulaItemView>,
    pub source_default_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct FormulaItemView {
    pub id: i64,
    pub dye_name: String,
    pub dye_code: Option<String>,
    pub amount: f64,
    pub unit: String,
    pub sort_order: u16,
}

impl From<&DefaultFormula> for FormulaView {
    fn from(f: &DefaultFormula) -> Self {
        Self {
            id: f.id().expect("persisted").value(),
            internal_color_code: <DefaultFormula as CalculableFormula>::internal_color_code(f)
                .as_str()
                .to_owned(),
            customer_color_code: f.customer_color_code().map(|c| c.as_str().to_owned()),
            color_name: f.color_name().map(str::to_owned),
            description: f.description().map(str::to_owned),
            base_weight_kg: f.base_weight_kg().map(|k| k.value()),
            liquor_ratio: <DefaultFormula as CalculableFormula>::liquor_ratio(f).map(|r| r.value()),
            notes: f.notes().map(str::to_owned),
            items: <DefaultFormula as CalculableFormula>::items(f)
                .iter()
                .map(item_to_view)
                .collect(),
            source_default_id: None,
            created_at: f.created_at(),
            updated_at: f.updated_at(),
        }
    }
}

impl From<&WorkspaceFormula> for FormulaView {
    fn from(f: &WorkspaceFormula) -> Self {
        Self {
            id: f.id().expect("persisted").value(),
            internal_color_code: <WorkspaceFormula as CalculableFormula>::internal_color_code(f)
                .as_str()
                .to_owned(),
            customer_color_code: f.customer_color_code().map(|c| c.as_str().to_owned()),
            color_name: f.color_name().map(str::to_owned),
            description: f.description().map(str::to_owned),
            base_weight_kg: f.base_weight_kg().map(|k| k.value()),
            liquor_ratio: <WorkspaceFormula as CalculableFormula>::liquor_ratio(f)
                .map(|r| r.value()),
            notes: f.notes().map(str::to_owned),
            items: <WorkspaceFormula as CalculableFormula>::items(f)
                .iter()
                .map(item_to_view)
                .collect(),
            source_default_id: f.source_default_id().map(|i| i.value()),
            created_at: f.created_at(),
            updated_at: f.updated_at(),
        }
    }
}

fn item_to_view(item: &crate::domain::formula::formula_item::FormulaItem) -> FormulaItemView {
    FormulaItemView {
        id: item.id().map(|i| i.value()).unwrap_or(0),
        dye_name: item.dye_name().to_owned(),
        dye_code: item.dye_code().map(str::to_owned),
        amount: item.amount_value(),
        unit: item.unit().as_db_str().to_owned(),
        sort_order: item.sort_order(),
    }
}

// ---------- Calculation ----------

#[derive(Debug, Deserialize)]
pub struct CalculateCmd {
    pub internal_color_code: String,
    pub target_kg: f64,
}

#[derive(Debug, Serialize)]
pub struct CalculationLineView {
    pub dye_name: String,
    pub dye_code: Option<String>,
    pub grams: f64,
    pub unit_used: String,
}

impl From<&CalculationLine> for CalculationLineView {
    fn from(l: &CalculationLine) -> Self {
        Self {
            dye_name: l.dye_name.clone(),
            dye_code: l.dye_code.clone(),
            grams: l.grams.value(),
            unit_used: l.unit_used.as_db_str().to_owned(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CalculationResultView {
    pub source: String,
    pub source_label: String,
    pub formula_id: Option<i64>,
    pub internal_color_code: String,
    pub target_kg: f64,
    pub lines: Vec<CalculationLineView>,
}

impl From<&CalculationResult> for CalculationResultView {
    fn from(r: &CalculationResult) -> Self {
        Self {
            source: match r.source {
                FormulaSource::CurrentWorkspace => "current_workspace".into(),
                FormulaSource::DefaultFallback => "default_fallback".into(),
            },
            source_label: r.source.display_label().to_owned(),
            formula_id: r.formula_id.map(|i| i.value()),
            internal_color_code: r.internal_color_code.as_str().to_owned(),
            target_kg: r.target_kg.value(),
            lines: r.lines.iter().map(CalculationLineView::from).collect(),
        }
    }
}

// ---------- Cart ----------

#[derive(Debug, Deserialize)]
pub struct AddToCartCmd {
    pub source_kind: String,
    pub source_formula_id: i64,
    pub target_kg: f64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCartKgCmd {
    pub source_kind: String,
    pub source_formula_id: i64,
    pub target_kg: f64,
}

#[derive(Debug, Deserialize)]
pub struct RemoveFromCartCmd {
    pub source_kind: String,
    pub source_formula_id: i64,
}

#[derive(Debug, Serialize)]
pub struct CartLineView {
    pub source_kind: String,
    pub source_formula_id: i64,
    pub target_kg: f64,
    pub added_at: DateTime<Utc>,
    pub internal_color_code: Option<String>,
    pub color_name: Option<String>,
    pub customer_color_code: Option<String>,
    pub calculation: Option<CalculationResultView>,
    pub error: Option<String>,
}

impl CartLineView {
    pub fn from_app(line: &crate::application::cart::CartLine) -> Self {
        let item: &CartItem = &line.item;
        Self {
            source_kind: item.source_kind().as_db_str().to_owned(),
            source_formula_id: item.source_formula_id().value(),
            target_kg: item.target_kg().value(),
            added_at: item.added_at(),
            internal_color_code: line.internal_color_code.clone(),
            color_name: line.color_name.clone(),
            customer_color_code: line.customer_color_code.clone(),
            calculation: line.calculation.as_ref().ok().map(CalculationResultView::from),
            error: line.calculation.as_ref().err().cloned(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ExportCartCmd {
    pub format: String, // "csv" | "pdf"
    pub out_path: String,
}

// ---------- Backup ----------

#[derive(Debug, Deserialize)]
pub struct ExportBackupCmd {
    pub passphrase: String,
    pub out_path: String,
}

#[derive(Debug, Deserialize)]
pub struct ImportBackupCmd {
    pub passphrase: String,
    pub in_path: String,
}

// ---------- Audit ----------

#[derive(Debug, Deserialize)]
pub struct ListAuditCmd {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub user_ids: Option<Vec<i64>>,
    pub actions: Option<Vec<String>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ExportAuditCmd {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub user_ids: Option<Vec<i64>>,
    pub actions: Option<Vec<String>>,
    pub format: String, // "encrypted" | "csv"
    pub passphrase: Option<String>,
    pub out_path: String,
}

#[derive(Debug, Serialize)]
pub struct AuditEventView {
    pub id: i64,
    pub event_uuid: String,
    pub user_id: Option<i64>,
    pub workspace_context_id: Option<i64>,
    pub action: String,
    pub target: Option<String>,
    pub details: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

impl From<&AuditEvent> for AuditEventView {
    fn from(e: &AuditEvent) -> Self {
        Self {
            id: e.id().expect("persisted").value(),
            event_uuid: e.event_uuid().to_string(),
            user_id: e.user_id().map(|i| i.value()),
            workspace_context_id: e.workspace_context_id().map(|i| i.value()),
            action: e.action().as_db_str().to_owned(),
            target: e.target().map(str::to_owned),
            details: e.details().map(str::to_owned),
            occurred_at: e.occurred_at(),
        }
    }
}

// ---------- Boot status ----------

#[derive(Debug, Serialize)]
pub struct BootStatusView {
    pub keystore_exists: bool,
    pub db_initialized: bool,
    pub user_count: u64,
}

// ---------- Helper: 角色解析 ----------

pub fn parse_role(s: &str) -> Result<Role, String> {
    match s {
        "admin" => Ok(Role::Admin),
        "user" => Ok(Role::User),
        other => Err(format!("未知的角色：{other}")),
    }
}
