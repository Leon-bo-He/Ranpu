//! 从 .ranpu 文件解密 + 反序列化 + 写入当前工作区配方.
//!
//! 同 (workspace_id, internal_color_code) 已存在 → 跳过 (status="skipped_duplicate");
//! 解析/校验失败 → 记错继续 (status="failed"); 否则插入 (status="imported").

use std::path::PathBuf;
use std::str::FromStr;

use crate::application::errors::{AppError, AppResult};
use crate::application::formula::import_default_formulas::{
    ImportFormulasSummary, ImportItemOutcome, ImportItemStatus,
};
use crate::application::formula::service::FormulaService;
use crate::application::formula::wire::{
    FormulaExportFile, FormulaExportItem, FORMULA_EXPORT_MAGIC, FORMULA_EXPORT_VERSION,
};
use crate::application::session_guard::{ensure_active_workspace, ensure_admin};
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::formula::amounts::Kilograms;
use crate::domain::formula::customer_color_code::CustomerColorCode;
use crate::domain::formula::formula_item::FormulaItem;
use crate::domain::formula::internal_color_code::InternalColorCode;
use crate::domain::formula::liquor_ratio::LiquorRatio;
use crate::domain::formula::unit::Unit;
use crate::domain::formula::workspace_formula::WorkspaceFormula;
use crate::domain::shared::id::WorkspaceId;

#[derive(Debug, Clone)]
pub struct ImportWorkspaceFormulasInput {
    pub passphrase: String,
    pub in_path: PathBuf,
}

impl FormulaService {
    pub fn import_workspace_formulas(
        &self,
        input: ImportWorkspaceFormulasInput,
    ) -> AppResult<ImportFormulasSummary> {
        let snap = ensure_admin(&*self.session_store)?;
        let (_, workspace_id) = ensure_active_workspace(&*self.session_store)?;
        let now = self.clock.now();

        let bytes = self
            .encrypted_importer
            .import_from_file(&input.in_path, &input.passphrase)
            .map_err(|e| AppError::Crypto(e.to_string()))?;

        let payload: FormulaExportFile = serde_json::from_slice(&bytes)
            .map_err(|e| AppError::Internal(format!("解析配方导出 JSON 失败：{e}")))?;
        if payload.magic != FORMULA_EXPORT_MAGIC {
            return Err(AppError::Internal(
                "文件签名不匹配，可能不是染谱配方导出文件".into(),
            ));
        }
        if payload.version != FORMULA_EXPORT_VERSION {
            return Err(AppError::Internal(format!(
                "不支持的导出文件版本：{}",
                payload.version
            )));
        }
        // payload.scope 仅作为元数据展示, 不强制 workspace 来源:
        // 默认库导出的 .ranpu 也允许导入到工作区.

        let mut items = Vec::with_capacity(payload.formulas.len());
        let mut imported = 0_u32;
        let mut skipped = 0_u32;
        let mut failed = 0_u32;
        for wire in payload.formulas {
            let internal_str = wire.internal_color_code.clone();
            let outcome = match self.import_one_workspace(&wire, workspace_id, now) {
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
            items.push(outcome);
        }

        let event = AuditEvent::new(
            Some(snap.user_id()),
            Some(workspace_id),
            Action::WorkspaceFormulasImported,
            Some(input.in_path.to_string_lossy().into_owned()),
            Some(format!(
                "imported={imported};skipped={skipped};failed={failed}"
            )),
            now,
        );
        self.audit_writer.record(&event)?;

        Ok(ImportFormulasSummary {
            items,
            imported,
            skipped,
            failed,
        })
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
        let base_kg = match wire.base_weight_kg {
            Some(v) => Some(Kilograms::new(v)?),
            None => None,
        };
        let ratio = match wire.liquor_ratio {
            Some(v) => Some(LiquorRatio::new(v)?),
            None => None,
        };
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
            wire.color_name.clone(),
            wire.description.clone(),
            base_kg,
            ratio,
            wire.notes.clone(),
            items,
            None,
            now,
        )?;
        self.workspace_repo.upsert(&formula)?;
        Ok(true)
    }
}
