//! 把默认配方库的若干条配方加密导出为 .ydaexp 文件，便于跨机分发。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::application::errors::{AppError, AppResult};
use crate::application::formula::service::FormulaService;
use crate::application::ports::default_formula_repository::DefaultFormulaQuery;
use crate::application::session_guard::ensure_admin;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::calculation::dye_calculator::CalculableFormula;
use crate::domain::formula::default_formula::DefaultFormula;
use crate::domain::shared::id::FormulaId;

pub const FORMULA_EXPORT_MAGIC: &str = "ranpu-formula-export";
pub const FORMULA_EXPORT_VERSION: u32 = 1;

#[derive(Debug, Clone)]
pub struct ExportDefaultFormulasInput {
    /// 空 → 导出当前默认库的全部配方; 非空 → 只导出指定 id.
    pub default_formula_ids: Vec<FormulaId>,
    pub passphrase: String,
    pub out_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct FormulaExportFile {
    pub magic: String,
    pub version: u32,
    pub exported_at: String,
    pub scope: String,
    pub formulas: Vec<FormulaExportItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct FormulaExportItem {
    pub internal_color_code: String,
    pub customer_color_code: Option<String>,
    pub color_name: Option<String>,
    pub description: Option<String>,
    pub base_weight_kg: Option<f64>,
    pub liquor_ratio: Option<f64>,
    pub notes: Option<String>,
    pub items: Vec<FormulaExportItemDye>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct FormulaExportItemDye {
    pub dye_name: String,
    pub dye_code: Option<String>,
    pub amount: f64,
    pub unit: String,
    pub sort_order: u16,
}

impl FormulaService {
    pub fn export_default_formulas(
        &self,
        input: ExportDefaultFormulasInput,
    ) -> AppResult<u32> {
        let snap = ensure_admin(&*self.session_store)?;
        if input.passphrase.chars().count() < 8 {
            return Err(AppError::Internal("加密导出口令至少 8 位".into()));
        }

        // 取要导出的配方
        let formulas: Vec<DefaultFormula> = if input.default_formula_ids.is_empty() {
            self.default_repo.list(DefaultFormulaQuery {
                keyword: None,
                limit: None,
                offset: None,
            })?
        } else {
            let mut out = Vec::with_capacity(input.default_formula_ids.len());
            for id in &input.default_formula_ids {
                if let Some(f) = self.default_repo.find_by_id(*id)? {
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
            scope: "default".into(),
            formulas: formulas.iter().map(formula_to_wire).collect(),
        };

        let bytes = serde_json::to_vec(&payload)
            .map_err(|e| AppError::Internal(format!("序列化配方导出 JSON 失败：{e}")))?;
        self.encrypted_exporter
            .export_to_file(&bytes, &input.passphrase, &input.out_path)
            .map_err(|e| AppError::Crypto(e.to_string()))?;

        let count = formulas.len() as u32;
        let event = AuditEvent::new(
            Some(snap.user_id()),
            None,
            Action::DefaultFormulasExported,
            Some(input.out_path.to_string_lossy().into_owned()),
            Some(format!("count={count}")),
            self.clock.now(),
        );
        self.audit_writer.record(&event)?;
        Ok(count)
    }
}

pub(super) fn formula_to_wire(f: &DefaultFormula) -> FormulaExportItem {
    FormulaExportItem {
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
            .map(|it| FormulaExportItemDye {
                dye_name: it.dye_name().to_owned(),
                dye_code: it.dye_code().map(str::to_owned),
                amount: it.amount_value(),
                unit: it.unit().as_db_str().to_owned(),
                sort_order: it.sort_order(),
            })
            .collect(),
    }
}
