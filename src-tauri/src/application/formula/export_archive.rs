//! 聚合配方库归档导出: 默认库 + 任意工作区一次打包成单个 .ranpu 文件.

use std::path::PathBuf;

use crate::application::errors::{AppError, AppResult};
use crate::application::formula::service::FormulaService;
use crate::application::formula::wire::{
    default_to_wire, workspace_to_wire, FormulaArchive, WorkspaceArchive,
    FORMULA_ARCHIVE_MAGIC, FORMULA_ARCHIVE_VERSION,
};
use crate::application::ports::default_formula_repository::DefaultFormulaQuery;
use crate::application::ports::workspace_formula_repository::WorkspaceFormulaQuery;
use crate::application::session_guard::ensure_active;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::shared::id::WorkspaceId;

#[derive(Debug, Clone)]
pub struct ExportArchiveInput {
    /// 是否包含默认配方库.
    pub include_default: bool,
    /// 要导出的工作区 id 列表 (空则不导出工作区).
    pub workspace_ids: Vec<WorkspaceId>,
    pub passphrase: String,
    pub out_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ExportArchiveSummary {
    pub default_count: u32,
    pub workspace_count: u32,
    pub workspace_formula_count: u32,
}

impl FormulaService {
    pub fn export_library_archive(
        &self,
        input: ExportArchiveInput,
    ) -> AppResult<ExportArchiveSummary> {
        let _ = ensure_active(&*self.session_store)?;
        if input.passphrase.chars().count() < 8 {
            return Err(AppError::Internal("加密导出口令至少 8 位".into()));
        }
        if !input.include_default && input.workspace_ids.is_empty() {
            return Err(AppError::Internal("至少选择默认库或一个工作区".into()));
        }

        let default_formulas = if input.include_default {
            self.default_repo
                .list(DefaultFormulaQuery {
                    keyword: None,
                    limit: None,
                    offset: None,
                })?
                .iter()
                .map(default_to_wire)
                .collect()
        } else {
            Vec::new()
        };

        let mut workspaces = Vec::with_capacity(input.workspace_ids.len());
        let mut workspace_formula_count = 0_u32;
        for ws_id in &input.workspace_ids {
            let ws = self
                .workspaces_repo
                .find_by_id(*ws_id)?
                .ok_or_else(|| AppError::Internal(format!("工作区不存在: {ws_id:?}")))?;
            let formulas = self.workspace_repo.list(WorkspaceFormulaQuery {
                workspace_id: *ws_id,
                keyword: None,
                limit: None,
                offset: None,
            })?;
            workspace_formula_count += formulas.len() as u32;
            workspaces.push(WorkspaceArchive {
                name: ws.name().as_str().to_owned(),
                description: ws.description().map(str::to_owned),
                formulas: formulas.iter().map(workspace_to_wire).collect(),
            });
        }

        let payload = FormulaArchive {
            magic: FORMULA_ARCHIVE_MAGIC.into(),
            version: FORMULA_ARCHIVE_VERSION,
            exported_at: self.clock.now().to_rfc3339(),
            default_formulas,
            workspaces,
        };
        let default_count = payload.default_formulas.len() as u32;
        let workspace_count = payload.workspaces.len() as u32;

        let bytes = serde_json::to_vec(&payload)
            .map_err(|e| AppError::Internal(format!("序列化配方归档 JSON 失败：{e}")))?;
        self.encrypted_exporter
            .export_to_file(&bytes, &input.passphrase, &input.out_path)
            .map_err(|e| AppError::Crypto(e.to_string()))?;

        let event = AuditEvent::new(
            None,
            Action::LibraryArchiveExported,
            Some(input.out_path.to_string_lossy().into_owned()),
            Some(format!(
                "default={default_count};workspaces={workspace_count};ws_formulas={workspace_formula_count}"
            )),
            self.clock.now(),
        );
        self.audit_writer.record(&event)?;
        Ok(ExportArchiveSummary {
            default_count,
            workspace_count,
            workspace_formula_count,
        })
    }
}
