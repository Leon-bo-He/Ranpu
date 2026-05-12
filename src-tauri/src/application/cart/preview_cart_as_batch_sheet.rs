use crate::application::cart::service::CartService;
use crate::application::errors::{AppError, AppResult};
use crate::application::ports::batch_sheet_exporter::{
    BatchSheetContext, BatchSheetFormat, FormulaMeta, YarnEntry,
};
use crate::application::session_guard::ensure_active_workspace;

#[derive(Debug, Clone, Default)]
pub struct PreviewBatchSheetInput {
    /// 客户名 (打印头). None 则 fallback 到当前工作区名称.
    pub customer: Option<String>,
    /// 每条 cart line 一份元信息: 单个 vat (整组共用) + 多条纱支变体.
    /// 长度可短于 cart, 缺位按空 meta 兜底.
    pub per_formula: Vec<PreviewFormulaMetaInput>,
    /// 渲染版本: Standard = 经典每条配方一段; Grid = A4 四宫格;
    /// A6Punch = A6 穿孔纸 一条一张. 默认 A6Punch (车间主用).
    pub layout: PreviewLayout,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PreviewLayout {
    Standard,
    Grid,
    #[default]
    A6Punch,
    Label,
}

#[derive(Debug, Clone, Default)]
pub struct PreviewFormulaMetaInput {
    pub vat_number: Option<String>,
    pub yarns: Vec<PreviewYarnEntryInput>,
}

#[derive(Debug, Clone, Default)]
pub struct PreviewYarnEntryInput {
    pub mill: Option<String>,
    pub spec: Option<String>,
    pub count: Option<String>,
}

/// 把 String 字段 trim 后空串 → None. 渲染层只看 Some, 不需要再判空.
fn norm(s: Option<String>) -> Option<String> {
    s.and_then(|v| {
        let t = v.trim();
        if t.is_empty() { None } else { Some(t.to_owned()) }
    })
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

        // 只渲染能算出结果的行, 失败行留给批次清单页面提示用户先修配方.
        // 同一个 cart line 一份配方 + 一组共用 vat + N 条纱支变体, 渲染时
        // 这一格 / 段里把多条变体作为内联多行展示, 不再展开成多个独立单元.
        let mut results = Vec::new();
        struct OwnedMeta {
            color_family: Option<String>,
            vat: Option<String>,
            yarns: Vec<(Option<String>, Option<String>, Option<String>)>,
        }
        let mut metas_owned: Vec<OwnedMeta> = Vec::new();
        for (idx, line) in lines.into_iter().enumerate() {
            if line.calculation.is_err() {
                continue;
            }
            let family = line.color_family.clone();
            let calc = line.calculation.expect("checked above");
            results.push(calc);
            let meta_in = input.per_formula.get(idx).cloned().unwrap_or_default();
            let yarns: Vec<_> = meta_in
                .yarns
                .into_iter()
                .map(|y| (norm(y.mill), norm(y.spec), norm(y.count)))
                // 整条 (mill, spec, count) 都为空就不渲染这一行.
                .filter(|(m, s, c)| m.is_some() || s.is_some() || c.is_some())
                .collect();
            metas_owned.push(OwnedMeta {
                color_family: family,
                vat: norm(meta_in.vat_number),
                yarns,
            });
        }
        let metas: Vec<FormulaMeta<'_>> = metas_owned
            .iter()
            .map(|m| FormulaMeta {
                color_family: m.color_family.as_deref(),
                vat_number: m.vat.as_deref(),
                yarns: m
                    .yarns
                    .iter()
                    .map(|(mill, spec, count)| YarnEntry {
                        mill: mill.as_deref(),
                        spec: spec.as_deref(),
                        count: count.as_deref(),
                    })
                    .collect(),
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
            PreviewLayout::A6Punch => BatchSheetFormat::HtmlA6Punch,
            PreviewLayout::Label => BatchSheetFormat::HtmlLabel,
        };
        self.batch_sheet_exporter
            .render(&results, context, format)
            .map_err(|e| AppError::Io(e.to_string()))
    }
}
