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

/// 批次单上下文 (顶部标题区域引用).
#[derive(Debug, Clone, Default)]
pub struct BatchSheetContext<'a> {
    /// 当前工作区名称, 业务上等同于客户名 (HTML 头部「客户」字段).
    /// 用户在预览/打印对话框可以改写这个值 (默认填的是工作区名).
    pub workspace_name: Option<&'a str>,
    /// 缸号 (例如 "5-2" = 第 5 缸第 2 批), HTML 头部「缸号」字段; None 则不显示.
    pub vat_number: Option<&'a str>,
    /// 纱支 (例如 "32S/2"), HTML 头部「纱支」字段; None 则不显示.
    pub yarn_count: Option<&'a str>,
}

/// 把购物车的多条计算结果导出为「批次单」文件（PROMPT 第 297 行）。
pub trait BatchSheetExporter: Send + Sync {
    fn export(
        &self,
        results: &[CalculationResult],
        context: BatchSheetContext<'_>,
        format: BatchSheetFormat,
        out_path: &Path,
    ) -> Result<(), BatchSheetError>;

    /// 渲染但不落盘. 用于 UI 内 iframe 预览 / 打印, 不需要让用户先选保存路径.
    fn render(
        &self,
        results: &[CalculationResult],
        context: BatchSheetContext<'_>,
        format: BatchSheetFormat,
    ) -> Result<String, BatchSheetError>;
}
