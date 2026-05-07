use std::path::Path;

use thiserror::Error;

use crate::domain::calculation::dye_calculator::CalculationResult;

#[derive(Debug, Error)]
pub enum BatchSheetError {
    #[error("文件读写错误：{0}")]
    Io(String),
    #[error("批次单生成失败：{0}")]
    Render(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchSheetFormat {
    Csv,
    Pdf,
}

/// 把购物车的多条计算结果导出为「批次单」文件（PROMPT 第 297 行）。
pub trait BatchSheetExporter: Send + Sync {
    fn export(
        &self,
        results: &[CalculationResult],
        format: BatchSheetFormat,
        out_path: &Path,
    ) -> Result<(), BatchSheetError>;
}
