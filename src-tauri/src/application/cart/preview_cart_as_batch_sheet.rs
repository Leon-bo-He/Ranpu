use crate::application::cart::service::CartService;
use crate::application::errors::{AppError, AppResult};
use crate::application::ports::batch_sheet_exporter::{BatchSheetContext, BatchSheetFormat};
use crate::application::session_guard::ensure_active_workspace;

impl CartService {
    /// 渲染当前批次清单为 HTML 字符串, 用于前端 iframe 预览 + 打印.
    /// 不落盘, 不写审计 (纯渲染, 用户的"打印"动作我们看不到, 没法
    /// 客观记录, 不如不假装记录).
    pub fn preview_cart_as_batch_sheet_html(&self) -> AppResult<String> {
        let (_snap, workspace_id) = ensure_active_workspace(&*self.session_store)?;
        let lines = self.list_cart_with_calculations()?;

        // 复用 export 流程的过滤策略: 只渲染能算出结果的行, 失败行
        // 留给批次清单页面提示用户先修配方.
        let results: Vec<_> = lines.into_iter().filter_map(|l| l.calculation.ok()).collect();

        let workspace_name = self
            .workspaces_repo
            .find_by_id(workspace_id)?
            .map(|w| w.name().as_str().to_owned());
        let context = BatchSheetContext {
            workspace_name: workspace_name.as_deref(),
        };

        self.batch_sheet_exporter
            .render(&results, context, BatchSheetFormat::Html)
            .map_err(|e| AppError::Io(e.to_string()))
    }
}
