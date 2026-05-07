//! 把当前工作区的若干条配方加密导出为 .ranpu 文件。

use std::path::PathBuf;

use crate::application::errors::{AppError, AppResult};
use crate::application::formula::service::FormulaService;
use crate::application::formula::wire::{
    workspace_to_wire, FormulaExportFile, FORMULA_EXPORT_MAGIC, FORMULA_EXPORT_VERSION,
};
use crate::application::ports::workspace_formula_repository::WorkspaceFormulaQuery;
use crate::application::session_guard::{ensure_active_workspace, ensure_admin};
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::formula::workspace_formula::WorkspaceFormula;
use crate::domain::shared::id::FormulaId;

#[derive(Debug, Clone)]
pub struct ExportWorkspaceFormulasInput {
    /// 空 → 导出当前工作区的全部配方; 非空 → 只导出指定 id (限当前工作区).
    pub workspace_formula_ids: Vec<FormulaId>,
    pub passphrase: String,
    pub out_path: PathBuf,
}

impl FormulaService {
    pub fn export_workspace_formulas(
        &self,
        input: ExportWorkspaceFormulasInput,
    ) -> AppResult<u32> {
        let snap = ensure_admin(&*self.session_store)?;
        let (_, workspace_id) = ensure_active_workspace(&*self.session_store)?;
        if input.passphrase.chars().count() < 8 {
            return Err(AppError::Internal("加密导出口令至少 8 位".into()));
        }

        let formulas: Vec<WorkspaceFormula> = if input.workspace_formula_ids.is_empty() {
            self.workspace_repo.list(WorkspaceFormulaQuery {
                workspace_id,
                keyword: None,
                limit: None,
                offset: None,
            })?
        } else {
            let mut out = Vec::with_capacity(input.workspace_formula_ids.len());
            for id in &input.workspace_formula_ids {
                if let Some(f) = self.workspace_repo.find_by_id(workspace_id, *id)? {
                    out.push(f);
                }
            }
            out
        };

        if formulas.is_empty() {
            return Err(AppError::Internal("没有匹配到任何配方可导出".into()));
        }

        let payload = FormulaExportFile {
            magic: FORMULA_EXPORT_MAGIC.into(),
            version: FORMULA_EXPORT_VERSION,
            exported_at: self.clock.now().to_rfc3339(),
            scope: "workspace".into(),
            formulas: formulas.iter().map(workspace_to_wire).collect(),
        };

        let bytes = serde_json::to_vec(&payload)
            .map_err(|e| AppError::Internal(format!("序列化配方导出 JSON 失败：{e}")))?;
        self.encrypted_exporter
            .export_to_file(&bytes, &input.passphrase, &input.out_path)
            .map_err(|e| AppError::Crypto(e.to_string()))?;

        let count = formulas.len() as u32;
        let event = AuditEvent::new(
            Some(snap.user_id()),
            Some(workspace_id),
            Action::WorkspaceFormulasExported,
            Some(input.out_path.to_string_lossy().into_owned()),
            Some(format!("count={count}")),
            self.clock.now(),
        );
        self.audit_writer.record(&event)?;
        Ok(count)
    }
}
