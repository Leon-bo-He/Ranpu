//! 从 .ranpu 文件解密 + 反序列化 + 写回默认配方库。
//!
//! 同内部色号已存在 → 跳过 (status="skipped_duplicate"); 解析/校验失败 →
//! 记错继续 (status="failed"); 否则插入新条目 (status="imported").

use std::path::PathBuf;
use std::str::FromStr;

use crate::application::errors::{AppError, AppResult};
use crate::application::formula::export_default_formulas::{
    FormulaExportFile, FORMULA_EXPORT_MAGIC, FORMULA_EXPORT_VERSION,
};
use crate::application::formula::service::FormulaService;
use crate::application::session_guard::ensure_admin;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::formula::amounts::Kilograms;
use crate::domain::formula::customer_color_code::CustomerColorCode;
use crate::domain::formula::default_formula::DefaultFormula;
use crate::domain::formula::formula_item::FormulaItem;
use crate::domain::formula::internal_color_code::InternalColorCode;
use crate::domain::formula::liquor_ratio::LiquorRatio;
use crate::domain::formula::unit::Unit;

#[derive(Debug, Clone)]
pub struct ImportDefaultFormulasInput {
    pub passphrase: String,
    pub in_path: PathBuf,
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
pub struct ImportFormulasSummary {
    pub items: Vec<ImportItemOutcome>,
    pub imported: u32,
    pub skipped: u32,
    pub failed: u32,
}

impl FormulaService {
    pub fn import_default_formulas(
        &self,
        input: ImportDefaultFormulasInput,
    ) -> AppResult<ImportFormulasSummary> {
        let snap = ensure_admin(&*self.session_store)?;
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
        if payload.scope != "default" {
            return Err(AppError::Internal(format!(
                "导出文件 scope=「{}」, 当前仅支持 default 配方库导入",
                payload.scope
            )));
        }

        let mut items = Vec::with_capacity(payload.formulas.len());
        let mut imported = 0_u32;
        let mut skipped = 0_u32;
        let mut failed = 0_u32;
        for wire in payload.formulas {
            let internal_str = wire.internal_color_code.clone();
            let outcome = match self.import_one(&wire, now) {
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
            None,
            Action::DefaultFormulasImported,
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

    /// 返回 Ok(true) 表示新插入, Ok(false) 表示因同内部色号存在跳过.
    fn import_one(
        &self,
        wire: &crate::application::formula::export_default_formulas::FormulaExportItem,
        now: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<bool> {
        let internal = InternalColorCode::new(wire.internal_color_code.clone())?;
        // 跳过重复
        if self.default_repo.find_by_internal_code(&internal)?.is_some() {
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
        let formula = DefaultFormula::new(
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
        self.default_repo.upsert(&formula)?;
        Ok(true)
    }
}
