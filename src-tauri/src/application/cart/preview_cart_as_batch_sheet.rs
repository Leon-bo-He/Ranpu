use crate::application::cart::service::CartService;
use crate::application::errors::{AppError, AppResult};
use crate::application::ports::batch_sheet_exporter::{
    BatchSheetContext, BatchSheetFormat, FormulaMeta,
};
use crate::application::session_guard::ensure_active_workspace;

#[derive(Debug, Clone, Default)]
pub struct PreviewBatchSheetInput {
    /// 客户名 (打印头). None 则 fallback 到当前工作区名称.
    pub customer: Option<String>,
    /// 每条购物车 line 对应一组元信息列表. 一个配方要打成多份不同纱支的
    /// 批次单时, 内层放多条 (每条独立的 vat / yarn). 内层空 vec 视为一
    /// 份空 meta. 顺序跟 list_cart_with_calculations 返回的 lines 一致.
    pub per_formula: Vec<Vec<PreviewFormulaMetaInput>>,
    /// 渲染版本: Standard = 经典每条配方一段; Grid = A4 四宫格.
    /// 默认 Standard.
    pub layout: PreviewLayout,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PreviewLayout {
    #[default]
    Standard,
    Grid,
}

#[derive(Debug, Clone, Default)]
pub struct PreviewFormulaMetaInput {
    pub vat_number: Option<String>,
    pub yarn_count: Option<String>,
}

impl CartService {
    /// 渲染当前批次清单为 HTML 字符串, 用于前端 iframe 预览 + 打印.
    /// 不落盘, 不写审计 (纯渲染, 用户的"打印"动作我们看不到, 没法
    /// 客观记录, 不如不假装记录).
    pub fn preview_cart_as_batch_sheet_html(
        &self,
        input: PreviewBatchSheetInput,
    ) -> AppResult<String> {
        let (_, workspace_id) = ensure_active_workspace(&*self.session_store)?;
        let lines = self.list_cart_with_calculations()?;

        // 复用 export 流程的过滤策略: 只渲染能算出结果的行, 失败行
        // 留给批次清单页面提示用户先修配方. 同一个 cart line 可能要打多份
        // 不同纱支 (per_formula[idx] 是个 Vec) — 把 (result, meta) 展开成
        // 平的输出, 同一条 result 复制 N 份, 每份配上各自的 meta.
        // 色系直接从 cart line 上拿 (用户不用在 prompt 里再填一次).
        let mut results = Vec::new();
        let mut metas_owned: Vec<(Option<String>, Option<String>, Option<String>)> = Vec::new();
        for (idx, line) in lines.into_iter().enumerate() {
            if line.calculation.is_err() {
                continue;
            }
            let family = line.color_family.clone();
            let calc = line.calculation.expect("checked above");
            // 取这条 line 的元信息列表; 缺失或空 vec 都视为 "一份空 meta"
            // 兜底, 保证配方至少渲染一次.
            let line_metas = input.per_formula.get(idx).cloned().unwrap_or_default();
            let line_metas = if line_metas.is_empty() {
                vec![PreviewFormulaMetaInput::default()]
            } else {
                line_metas
            };
            for meta in line_metas {
                let vat = meta.vat_number.filter(|s| !s.trim().is_empty());
                let yarn = meta.yarn_count.filter(|s| !s.trim().is_empty());
                results.push(calc.clone());
                metas_owned.push((family.clone(), vat, yarn));
            }
        }
        let metas: Vec<FormulaMeta<'_>> = metas_owned
            .iter()
            .map(|(f, v, y)| FormulaMeta {
                color_family: f.as_deref(),
                vat_number: v.as_deref(),
                yarn_count: y.as_deref(),
            })
            .collect();

        // customer 优先用前端传过来的覆写值; 没传则走当前工作区名兜底.
        let customer = match input.customer {
            Some(s) if !s.trim().is_empty() => Some(s),
            _ => self
                .workspaces_repo
                .find_by_id(workspace_id)?
                .map(|w| w.name().as_str().to_owned()),
        };

        let context = BatchSheetContext {
            workspace_name: customer.as_deref(),
            per_formula: &metas,
        };

        let format = match input.layout {
            PreviewLayout::Standard => BatchSheetFormat::Html,
            PreviewLayout::Grid => BatchSheetFormat::HtmlGrid,
        };
        self.batch_sheet_exporter
            .render(&results, context, format)
            .map_err(|e| AppError::Io(e.to_string()))
    }
}
