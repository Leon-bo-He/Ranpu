//! 与前端交互的所有 DTO（serde 序列化）。
//!
//! 命名约定：Cmd 是命令入参，View 是返回。所有时间字段输出 RFC3339 字符串，
//! 前端按 `YYYY-MM-DD HH:mm` 自行格式化。
//!
//! 单用户解锁模型: 没有 LoginCmd / ChangePasswordCmd / CreateUserCmd /
//! UserView / UnlockOutcomeView 之类 — 没有用户体系自然就没有这些 DTO.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::audit::audit_event::AuditEvent;
use crate::domain::calculation::dye_calculator::{
    CalculableFormula, CalculationLine, CalculationResult, FormulaSource,
};
use crate::domain::cart::cart_item::CartItem;
use crate::domain::formula::default_formula::DefaultFormula;
use crate::domain::formula::workspace_formula::WorkspaceFormula;
use crate::domain::session::Session;
use crate::domain::workspace::workspace::Workspace;

// ---------- Boot / 解锁 ----------

#[derive(Debug, Deserialize)]
pub struct BootAppCmd {
    pub boot_passphrase: String,
}

#[derive(Debug, Deserialize)]
pub struct UnlockSessionCmd {
    pub passphrase: String,
}

#[derive(Debug, Serialize)]
pub struct SessionView {
    pub active_workspace_id: Option<i64>,
    pub locked: bool,
    pub last_activity_at: DateTime<Utc>,
}

impl From<&Session> for SessionView {
    fn from(s: &Session) -> Self {
        Self {
            active_workspace_id: s.active_workspace_id().map(|i| i.value()),
            locked: s.is_locked(),
            last_activity_at: s.last_activity_at(),
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
pub struct UpdateWorkspaceDescriptionCmd {
    pub workspace_id: i64,
    pub description: Option<String>,
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
    /// "normal" | "system_mirror"
    pub kind: String,
}

impl From<&Workspace> for WorkspaceView {
    fn from(w: &Workspace) -> Self {
        Self {
            id: w.id().expect("persisted").value(),
            name: w.name().as_str().to_owned(),
            description: w.description().map(str::to_owned),
            created_at: w.created_at(),
            kind: w.kind().as_db_str().to_owned(),
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
    pub color_family: Option<String>,
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
    pub color_family: Option<String>,
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
            color_family: f.color_family().map(str::to_owned),
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
            color_family: f.color_family().map(str::to_owned),
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

// ---------- Formula 批量复制 ----------

#[derive(Debug, Deserialize)]
pub struct BatchCopyDefaultCmd {
    pub default_formula_ids: Vec<i64>,
}

// ---------- Library Archive (聚合默认库 + 工作区) ----------

#[derive(Debug, Deserialize)]
pub struct ExportLibraryArchiveCmd {
    pub include_default: bool,
    pub workspace_ids: Vec<i64>,
    pub passphrase: String,
    pub out_path: String,
}

#[derive(Debug, Serialize)]
pub struct ExportLibraryArchiveView {
    pub default_count: u32,
    pub workspace_count: u32,
    pub workspace_formula_count: u32,
}

impl From<&crate::application::formula::ExportArchiveSummary> for ExportLibraryArchiveView {
    fn from(s: &crate::application::formula::ExportArchiveSummary) -> Self {
        Self {
            default_count: s.default_count,
            workspace_count: s.workspace_count,
            workspace_formula_count: s.workspace_formula_count,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PreviewLibraryArchiveCmd {
    pub passphrase: String,
    pub in_path: String,
}

#[derive(Debug, Serialize)]
pub struct PreviewLibraryArchiveView {
    pub exported_at: String,
    pub default_count: u32,
    pub has_default: bool,
    pub workspaces: Vec<PreviewWorkspaceView>,
}

#[derive(Debug, Serialize)]
pub struct PreviewWorkspaceView {
    pub name: String,
    pub description: Option<String>,
    pub formula_count: u32,
    pub already_exists: bool,
}

impl From<&crate::application::formula::PreviewArchive> for PreviewLibraryArchiveView {
    fn from(p: &crate::application::formula::PreviewArchive) -> Self {
        Self {
            exported_at: p.exported_at.clone(),
            default_count: p.default_count,
            has_default: p.has_default,
            workspaces: p
                .workspaces
                .iter()
                .map(|w| PreviewWorkspaceView {
                    name: w.name.clone(),
                    description: w.description.clone(),
                    formula_count: w.formula_count,
                    already_exists: w.already_exists,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ImportLibraryArchiveCmd {
    pub passphrase: String,
    pub in_path: String,
    pub include_default: bool,
    pub workspace_plans: Vec<WorkspaceImportPlanDto>,
}

#[derive(Debug, Deserialize)]
pub struct WorkspaceImportPlanDto {
    pub name: String,
    /// "skip" | "merge" | "create_new"
    pub action: String,
}

#[derive(Debug, Serialize)]
pub struct ImportItemOutcomeView {
    pub internal_color_code: String,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ImportSectionSummaryView {
    pub items: Vec<ImportItemOutcomeView>,
    pub imported: u32,
    pub skipped: u32,
    pub failed: u32,
}

impl From<&crate::application::formula::ImportSectionSummary> for ImportSectionSummaryView {
    fn from(s: &crate::application::formula::ImportSectionSummary) -> Self {
        Self {
            items: s
                .items
                .iter()
                .map(|i| ImportItemOutcomeView {
                    internal_color_code: i.internal_color_code.clone(),
                    status: i.status.as_str().to_owned(),
                    error: i.error.clone(),
                })
                .collect(),
            imported: s.imported,
            skipped: s.skipped,
            failed: s.failed,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ImportWorkspaceSummaryView {
    pub name: String,
    pub action: String,
    pub summary: ImportSectionSummaryView,
}

#[derive(Debug, Serialize)]
pub struct ImportLibraryArchiveView {
    pub default_summary: Option<ImportSectionSummaryView>,
    pub workspace_summaries: Vec<ImportWorkspaceSummaryView>,
}

impl From<&crate::application::formula::ImportArchiveSummary> for ImportLibraryArchiveView {
    fn from(s: &crate::application::formula::ImportArchiveSummary) -> Self {
        Self {
            default_summary: s.default_summary.as_ref().map(ImportSectionSummaryView::from),
            workspace_summaries: s
                .workspace_summaries
                .iter()
                .map(|w| ImportWorkspaceSummaryView {
                    name: w.name.clone(),
                    action: w.action.clone(),
                    summary: ImportSectionSummaryView::from(&w.summary),
                })
                .collect(),
        }
    }
}

pub fn parse_workspace_import_action(
    s: &str,
) -> Result<crate::application::formula::WorkspaceImportAction, String> {
    use crate::application::formula::WorkspaceImportAction;
    match s {
        "skip" => Ok(WorkspaceImportAction::Skip),
        "merge" => Ok(WorkspaceImportAction::Merge),
        "create_new" => Ok(WorkspaceImportAction::CreateNew),
        other => Err(format!("未知的工作区导入动作：{other}")),
    }
}

#[derive(Debug, Serialize)]
pub struct BatchCopyOutcomeItemView {
    pub source_default_id: i64,
    pub new_workspace_formula_id: Option<i64>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BatchCopySummaryView {
    pub items: Vec<BatchCopyOutcomeItemView>,
    pub succeeded: u32,
    pub failed: u32,
}

impl From<&crate::application::formula::BatchCopySummary> for BatchCopySummaryView {
    fn from(s: &crate::application::formula::BatchCopySummary) -> Self {
        Self {
            items: s
                .items
                .iter()
                .map(|i| BatchCopyOutcomeItemView {
                    source_default_id: i.source_default_id.value(),
                    new_workspace_formula_id: i.new_workspace_formula_id.map(|i| i.value()),
                    error: i.error.clone(),
                })
                .collect(),
            succeeded: s.succeeded,
            failed: s.failed,
        }
    }
}

// ---------- Calculation ----------

#[derive(Debug, Deserialize)]
pub struct CalculateCmd {
    pub internal_color_code: String,
    pub target_kg: f64,
}

#[derive(Debug, Deserialize)]
pub struct SearchByCustomerCodeCmd {
    pub customer_color_code: String,
}

#[derive(Debug, Serialize)]
pub struct CustomerCodeMatchView {
    pub source: String,
    pub source_label: String,
    pub formula_id: Option<i64>,
    pub internal_color_code: String,
    pub color_family: Option<String>,
    pub customer_color_code: Option<String>,
}

impl From<&crate::application::calculation::CustomerCodeMatch> for CustomerCodeMatchView {
    fn from(m: &crate::application::calculation::CustomerCodeMatch) -> Self {
        Self {
            source: match m.source {
                FormulaSource::CurrentWorkspace => "current_workspace".into(),
                FormulaSource::DefaultFallback => "default_fallback".into(),
            },
            source_label: m.source.display_label().to_owned(),
            formula_id: m.formula_id.map(|i| i.value()),
            internal_color_code: m.internal_color_code.as_str().to_owned(),
            color_family: m.color_family.clone(),
            customer_color_code: m.customer_color_code.clone(),
        }
    }
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
    pub color_family: Option<String>,
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
            color_family: line.color_family.clone(),
            customer_color_code: line.customer_color_code.clone(),
            calculation: line.calculation.as_ref().ok().map(CalculationResultView::from),
            error: line.calculation.as_ref().err().cloned(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ExportCartCmd {
    pub format: String, // "csv" | "html"
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
    pub actions: Option<Vec<String>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ExportAuditCmd {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub actions: Option<Vec<String>>,
    pub format: String, // "encrypted" | "csv"
    pub passphrase: Option<String>,
    pub out_path: String,
}

#[derive(Debug, Serialize)]
pub struct AuditEventView {
    pub id: i64,
    pub event_uuid: String,
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
}
