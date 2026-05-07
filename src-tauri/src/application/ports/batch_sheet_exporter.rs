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
    /// CSV — 可在 Excel / WPS 直接打开做二次处理.
    Csv,
    /// HTML — 浏览器打开即可看; Ctrl+P → 另存为 PDF 满足打印 / 归档需求.
    /// 不直接生成 PDF 是因为要嵌 CJK 字体, 体积代价不划算.
    Html,
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
