//! 解密 + 解析 + 写入聚合归档.
//!
//! 调用方先调 preview_library_archive 拿到归档清单, 再针对每个工作区
//! 选择动作 (skip / merge into existing / create new), 并决定是否导入默认库,
//! 然后调用本用例.
//!
//! 同 (workspace_id, internal_color_code) 已存在时按 SkippedDuplicate;
//! 默认库同 internal_color_code 已存在时按 SkippedDuplicate; 解析失败按 Failed.

use std::path::PathBuf;
use std::str::FromStr;

use crate::application::errors::{AppError, AppResult};
use crate::application::formula::service::FormulaService;
use crate::application::formula::wire::{
    FormulaArchive, FormulaExportItem, WorkspaceArchive, FORMULA_ARCHIVE_MAGIC,
    FORMULA_ARCHIVE_VERSION,
};
use crate::application::session_guard::ensure_active;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::formula::customer_color_code::CustomerColorCode;
use crate::domain::formula::default_formula::DefaultFormula;
use crate::domain::formula::formula_item::FormulaItem;
use crate::domain::formula::internal_color_code::InternalColorCode;
use crate::domain::formula::unit::Unit;
use crate::domain::formula::workspace_formula::WorkspaceFormula;
use crate::domain::shared::id::WorkspaceId;
use crate::domain::workspace::workspace::{Workspace, WorkspaceName};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceImportAction {
    /// 跳过 — 不导入这个工作区的任何配方.
    Skip,
    /// 合并 — 当目标库存在同名工作区时, 把归档配方写入该工作区
    /// (同 internal_color_code 跳过).
    Merge,
    /// 新建 — 当目标库不存在同名工作区时, 创建后写入归档配方.
    CreateNew,
}

#[derive(Debug, Clone)]
pub struct WorkspaceImportPlan {
    /// 归档里的工作区名称, 与 preview 返回的 name 对应.
    pub name: String,
    pub action: WorkspaceImportAction,
}

#[derive(Debug, Clone)]
pub struct ImportArchiveInput {
    pub passphrase: String,
    pub in_path: PathBuf,
    pub include_default: bool,
    /// 每个工作区的处理决定; 未列出的工作区按 Skip 处理.
    pub workspace_plans: Vec<WorkspaceImportPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportItemStatus {
    Imported,
    SkippedDuplicate,
    Failed,
}

impl ImportItemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImportItemStatus::Imported => "imported",
            ImportItemStatus::SkippedDuplicate => "skipped_duplicate",
            ImportItemStatus::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImportItemOutcome {
    pub internal_color_code: String,
    pub status: ImportItemStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ImportSectionSummary {
    pub items: Vec<ImportItemOutcome>,
    pub imported: u32,
    pub skipped: u32,
    pub failed: u32,
}

#[derive(Debug, Clone)]
pub struct ImportWorkspaceSummary {
    pub name: String,
    /// "skipped" / "merged" / "created"
    pub action: String,
    pub summary: ImportSectionSummary,
}

#[derive(Debug, Clone)]
pub struct ImportArchiveSummary {
    pub default_summary: Option<ImportSectionSummary>,
    pub workspace_summaries: Vec<ImportWorkspaceSummary>,
}

impl FormulaService {
    pub fn import_library_archive(
        &self,
        input: ImportArchiveInput,
    ) -> AppResult<ImportArchiveSummary> {
        let _ = ensure_active(&*self.session_store)?;
        let now = self.clock.now();

        let bytes = self
            .encrypted_importer
            .import_from_file(&input.in_path, &input.passphrase)
            .map_err(|e| AppError::Crypto(e.to_string()))?;

        let payload: FormulaArchive = serde_json::from_slice(&bytes)
            .map_err(|e| AppError::Internal(format!("解析配方归档 JSON 失败：{e}")))?;
        if payload.magic != FORMULA_ARCHIVE_MAGIC {
            return Err(AppError::Internal(
                "文件签名不匹配，可能不是染谱配方导出文件".into(),
            ));
        }
        if payload.version != FORMULA_ARCHIVE_VERSION {
            return Err(AppError::Internal(format!(
                "不支持的归档版本：{}",
                payload.version
            )));
        }

        let default_summary = if input.include_default && !payload.default_formulas.is_empty() {
            Some(self.import_default_section(&payload.default_formulas, now)?)
        } else {
            None
        };

        let mut workspace_summaries = Vec::with_capacity(payload.workspaces.len());
        for ws in &payload.workspaces {
            let action = input
                .workspace_plans
                .iter()
                .find(|p| p.name == ws.name)
                .map(|p| p.action)
                .unwrap_or(WorkspaceImportAction::Skip);

            let (action_label, summary) = match action {
                WorkspaceImportAction::Skip => (
                    "skipped",
                    ImportSectionSummary {
                        items: Vec::new(),
                        imported: 0,
                        skipped: 0,
                        failed: 0,
                    },
                ),
                WorkspaceImportAction::Merge => {
                    let existing = self.workspaces_repo.find_by_name(&ws.name)?;
                    match existing {
                        Some(w) if w.is_system_mirror() => (
                            "skipped",
                            ImportSectionSummary {
                                items: Vec::new(),
                                imported: 0,
                                skipped: 0,
                                failed: 0,
                            },
                        ),
                        Some(w) => {
                            let id = w.id().ok_or_else(|| {
                                AppError::Internal(
                                    "目标工作区缺少 id, 数据库状态异常".into(),
                                )
                            })?;
                            ("merged", self.import_workspace_section(ws, id, now)?)
                        }
                        None => (
                            "created",
                            self.import_workspace_section(
                                ws,
                                self.create_workspace_for_import(ws, now)?,
                                now,
                            )?,
                        ),
                    }
                }
                WorkspaceImportAction::CreateNew => {
                    let existing = self.workspaces_repo.find_by_name(&ws.name)?;
                    match existing {
                        Some(_) => (
                            "skipped",
                            ImportSectionSummary {
                                items: Vec::new(),
                                imported: 0,
                                skipped: 0,
                                failed: 0,
                            },
                        ),
                        None => (
                            "created",
                            self.import_workspace_section(
                                ws,
                                self.create_workspace_for_import(ws, now)?,
                                now,
                            )?,
                        ),
                    }
                }
            };

            workspace_summaries.push(ImportWorkspaceSummary {
                name: ws.name.clone(),
                action: action_label.into(),
                summary,
            });
        }

        let total_default_imported = default_summary
            .as_ref()
            .map(|s| s.imported)
            .unwrap_or(0);
        let total_workspace_imported: u32 = workspace_summaries
            .iter()
            .map(|w| w.summary.imported)
            .sum();
        let workspaces_touched = workspace_summaries
            .iter()
            .filter(|w| w.action != "skipped")
            .count() as u32;

        let event = AuditEvent::new(
            None,
            Action::LibraryArchiveImported,
            Some(input.in_path.to_string_lossy().into_owned()),
            Some(format!(
                "default_imported={total_default_imported};workspaces_touched={workspaces_touched};ws_formulas_imported={total_workspace_imported}"
            )),
            now,
        );
        self.audit_writer.record(&event)?;

        Ok(ImportArchiveSummary {
            default_summary,
            workspace_summaries,
        })
    }

    fn create_workspace_for_import(
        &self,
        ws: &WorkspaceArchive,
        now: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<WorkspaceId> {
        let name = WorkspaceName::new(ws.name.clone())?;
        let workspace = Workspace::new(name, ws.description.clone(), now)?;
        let id = self.workspaces_repo.insert(&workspace)?;
        // 顺手补一笔工作区创建审计 (与正常 create_workspace 保持一致)
        let evt = AuditEvent::new(
            Some(id),
            Action::WorkspaceCreated,
            Some(ws.name.clone()),
            Some("via_archive_import".into()),
            now,
        );
        self.audit_writer.record(&evt)?;
        Ok(id)
    }

    fn import_default_section(
        &self,
        items: &[FormulaExportItem],
        now: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<ImportSectionSummary> {
        let mut outcomes = Vec::with_capacity(items.len());
        let (mut imported, mut skipped, mut failed) = (0u32, 0u32, 0u32);
        for wire in items {
            let internal_str = wire.internal_color_code.clone();
            let outcome = match self.import_one_default(wire, now) {
                Ok(true) => {
                    imported += 1;
                    ImportItemOutcome {
                        internal_color_code: internal_str,
                        status: ImportItemStatus::Imported,
                        error: None,
                    }
                }
                Ok(false) => {
                    skipped += 1;
                    ImportItemOutcome {
                        internal_color_code: internal_str,
                        status: ImportItemStatus::SkippedDuplicate,
                        error: None,
                    }
                }
                Err(e) => {
                    failed += 1;
                    ImportItemOutcome {
                        internal_color_code: internal_str,
                        status: ImportItemStatus::Failed,
                        error: Some(e.to_string()),
                    }
                }
            };
            outcomes.push(outcome);
        }
        Ok(ImportSectionSummary {
            items: outcomes,
            imported,
            skipped,
            failed,
        })
    }

    fn import_workspace_section(
        &self,
        ws: &WorkspaceArchive,
        workspace_id: WorkspaceId,
        now: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<ImportSectionSummary> {
        let mut outcomes = Vec::with_capacity(ws.formulas.len());
        let (mut imported, mut skipped, mut failed) = (0u32, 0u32, 0u32);
        for wire in &ws.formulas {
            let internal_str = wire.internal_color_code.clone();
            let outcome = match self.import_one_workspace(wire, workspace_id, now) {
                Ok(true) => {
                    imported += 1;
                    ImportItemOutcome {
                        internal_color_code: internal_str,
                        status: ImportItemStatus::Imported,
                        error: None,
                    }
                }
                Ok(false) => {
                    skipped += 1;
                    ImportItemOutcome {
                        internal_color_code: internal_str,
                        status: ImportItemStatus::SkippedDuplicate,
                        error: None,
                    }
                }
                Err(e) => {
                    failed += 1;
                    ImportItemOutcome {
                        internal_color_code: internal_str,
                        status: ImportItemStatus::Failed,
                        error: Some(e.to_string()),
                    }
                }
            };
            outcomes.push(outcome);
        }
        Ok(ImportSectionSummary {
            items: outcomes,
            imported,
            skipped,
            failed,
        })
    }

    fn import_one_default(
        &self,
        wire: &FormulaExportItem,
        now: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<bool> {
        let internal = InternalColorCode::new(wire.internal_color_code.clone())?;
        if self.default_repo.find_by_internal_code(&internal)?.is_some() {
            return Ok(false);
        }
        let customer = CustomerColorCode::maybe(wire.customer_color_code.clone())?;
        let mut items = Vec::with_capacity(wire.items.len());
        for it in &wire.items {
            let unit = Unit::from_str(&it.unit)?;
            items.push(FormulaItem::new(
                it.dye_name.clone(),
                it.dye_code.clone(),
                it.amount,
                unit,
                it.sort_order,
            )?);
        }
        let formula = DefaultFormula::new(
            internal,
            customer,
            wire.color_family.clone(),
            wire.notes.clone(),
            items,
            now,
        )?;
        self.default_repo.upsert(&formula)?;
        Ok(true)
    }

    fn import_one_workspace(
        &self,
        wire: &FormulaExportItem,
        workspace_id: WorkspaceId,
        now: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<bool> {
        let internal = InternalColorCode::new(wire.internal_color_code.clone())?;
        if self
            .workspace_repo
            .find_by_internal_code(workspace_id, &internal)?
            .is_some()
        {
            return Ok(false);
        }
        let customer = CustomerColorCode::maybe(wire.customer_color_code.clone())?;
        let mut items = Vec::with_capacity(wire.items.len());
        for it in &wire.items {
            let unit = Unit::from_str(&it.unit)?;
            items.push(FormulaItem::new(
                it.dye_name.clone(),
                it.dye_code.clone(),
                it.amount,
                unit,
                it.sort_order,
            )?);
        }
        let formula = WorkspaceFormula::new(
            workspace_id,
            internal,
            customer,
            wire.color_family.clone(),
            wire.notes.clone(),
            items,
            None,
            now,
        )?;
        self.workspace_repo.upsert(&formula)?;
        Ok(true)
    }
}
