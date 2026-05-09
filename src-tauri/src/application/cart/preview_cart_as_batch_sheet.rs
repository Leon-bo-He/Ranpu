use crate::application::cart::service::CartService;
use crate::application::errors::{AppError, AppResult};
use crate::application::ports::batch_sheet_exporter::{BatchSheetContext, BatchSheetFormat};
use crate::application::session_guard::ensure_active_workspace;

#[derive(Debug, Clone, Default)]
pub struct PreviewBatchSheetInput {
    /// 客户名 (打印头). None 则 fallback 到当前工作区名称.
    pub customer: Option<String>,
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
        // 留给批次清单页面提示用户先修配方.
        let results: Vec<_> = lines.into_iter().filter_map(|l| l.calculation.ok()).collect();

        // customer 优先用前端传过来的覆写值; 没传则走当前工作区名兜底.
        let customer = match input.customer {
            Some(s) if !s.trim().is_empty() => Some(s),
            _ => self
                .workspaces_repo
                .find_by_id(workspace_id)?
                .map(|w| w.name().as_str().to_owned()),
        };

        let vat_number = input
            .vat_number
            .filter(|s| !s.trim().is_empty());
        let yarn_count = input
            .yarn_count
            .filter(|s| !s.trim().is_empty());

        let context = BatchSheetContext {
            workspace_name: customer.as_deref(),
            vat_number: vat_number.as_deref(),
            yarn_count: yarn_count.as_deref(),
        };

        self.batch_sheet_exporter
            .render(&results, context, BatchSheetFormat::Html)
            .map_err(|e| AppError::Io(e.to_string()))
    }
}
