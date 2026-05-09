//! 加密 .ranpu 配方归档文件的 wire 格式 (聚合默认库 + 任意工作区).
//!
//! 文件 = AES-256-GCM 加密的 JSON, 内层就是这里定义的 FormulaArchive.
//!
//! 工作区在归档里以 `name` 作为主键 (而非数据库自增 id), 因为目标机器上
//! 同名工作区不一定存在 / id 不可能一致. 导入端按 name 在目标库匹配:
//!   - 找不到 → 新建工作区
//!   - 找到 → 由 UI 决定 merge / skip
//!
//! version 3 起精简字段: 砍 color_name / description / base_weight_kg /
//! liquor_ratio, 加 color_family. version 2 老归档仍可读 — 老字段在 wire
//! 上保留 #[serde(default)] 让旧文件解析不爆, 但导入时会丢失.

use serde::{Deserialize, Serialize};

use crate::domain::calculation::dye_calculator::CalculableFormula;
use crate::domain::formula::default_formula::DefaultFormula;
use crate::domain::formula::workspace_formula::WorkspaceFormula;

pub const FORMULA_ARCHIVE_MAGIC: &str = "ranpu-formula-export";
pub const FORMULA_ARCHIVE_VERSION: u32 = 3;

#[derive(Debug, Serialize, Deserialize)]
pub struct FormulaArchive {
    pub magic: String,
    pub version: u32,
    pub exported_at: String,
    /// 默认配方库导出条目; 不导出时为空 Vec.
    #[serde(default)]
    pub default_formulas: Vec<FormulaExportItem>,
    /// 工作区导出条目 (按工作区分组); 不导出时为空 Vec.
    #[serde(default)]
    pub workspaces: Vec<WorkspaceArchive>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceArchive {
    /// 工作区名称, 在目标机器上作为匹配主键.
    pub name: String,
    pub description: Option<String>,
    pub formulas: Vec<FormulaExportItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormulaExportItem {
    pub internal_color_code: String,
    pub customer_color_code: Option<String>,
    /// 1.0.7+ 字段; 旧归档没有, 默认 None.
    #[serde(default)]
    pub color_family: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<FormulaExportItemDye>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormulaExportItemDye {
    pub dye_name: String,
    pub dye_code: Option<String>,
    pub amount: f64,
    pub unit: String,
    pub sort_order: u16,
}

pub fn default_to_wire(f: &DefaultFormula) -> FormulaExportItem {
    FormulaExportItem {
        internal_color_code: <DefaultFormula as CalculableFormula>::internal_color_code(f)
            .as_str()
            .to_owned(),
        customer_color_code: f.customer_color_code().map(|c| c.as_str().to_owned()),
        color_family: f.color_family().map(str::to_owned),
        notes: f.notes().map(str::to_owned),
        items: <DefaultFormula as CalculableFormula>::items(f)
            .iter()
            .map(item_to_wire)
            .collect(),
    }
}

pub fn workspace_to_wire(f: &WorkspaceFormula) -> FormulaExportItem {
    FormulaExportItem {
        internal_color_code: <WorkspaceFormula as CalculableFormula>::internal_color_code(f)
            .as_str()
            .to_owned(),
        customer_color_code: f.customer_color_code().map(|c| c.as_str().to_owned()),
        color_family: f.color_family().map(str::to_owned),
        notes: f.notes().map(str::to_owned),
        items: <WorkspaceFormula as CalculableFormula>::items(f)
            .iter()
            .map(item_to_wire)
            .collect(),
    }
}

fn item_to_wire(it: &crate::domain::formula::formula_item::FormulaItem) -> FormulaExportItemDye {
    FormulaExportItemDye {
        dye_name: it.dye_name().to_owned(),
        dye_code: it.dye_code().map(str::to_owned),
        amount: it.amount_value(),
        unit: it.unit().as_db_str().to_owned(),
        sort_order: it.sort_order(),
    }
}
