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
    /// 标准 HTML — 每条配方一段, 头部 (h2 + 缸号 / 纱支 meta) + 染料表格.
    /// 浏览器打开即可看; Ctrl+P → 另存为 PDF.
    Html,
    /// 四宫格 HTML — A4 纸 2x2 格, 每格一条配方, 虚线分割.
    /// 染料 ≥ 8 种的配方独占跨 2 行 (整列), 不影响其他格.
    HtmlGrid,
}

/// 批次单上下文 (顶部标题区域引用).
#[derive(Debug, Clone, Default)]
pub struct BatchSheetContext<'a> {
    /// 当前工作区名称, 业务上等同于客户名 (HTML 头部「客户」字段).
    /// 用户在预览/打印对话框可以改写这个值 (默认填的是工作区名).
    pub workspace_name: Option<&'a str>,
    /// 每个配方独立的缸号 / 纱支元信息. 与 results 数组顺序对齐, 长度也应一致;
    /// 短了会按 None 兜底, 长了多余会忽略.
    pub per_formula: &'a [FormulaMeta<'a>],
}

/// 单条配方的额外元信息 — 渲染到该配方块的标题下方.
#[derive(Debug, Clone, Default)]
pub struct FormulaMeta<'a> {
    /// 配方的色系 (从 cart line / formula 取, 不需要用户填).
    pub color_family: Option<&'a str>,
    /// 缸号 (含缸次, 已拼好的 "X-Y" 格式). 同一配方下所有纱支变体共用.
    pub vat_number: Option<&'a str>,
    /// 纱支变体列表. 一个配方可能要在多种纱支上同时染, 每条变体一行
    /// "厂名 规格 个数". 空列表表示没填纱支, 渲染时跳过整段.
    pub yarns: Vec<YarnEntry<'a>>,
}

/// 单条纱支变体: 厂名 + 规格 + 个数. 各字段独立可空, 渲染时只输出非空段.
#[derive(Debug, Clone, Copy, Default)]
pub struct YarnEntry<'a> {
    pub mill: Option<&'a str>,
    pub spec: Option<&'a str>,
    pub count: Option<&'a str>,
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
