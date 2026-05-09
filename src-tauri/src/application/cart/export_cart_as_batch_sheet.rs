use std::path::PathBuf;

use crate::application::cart::service::CartService;
use crate::application::errors::{AppError, AppResult};
use crate::application::ports::batch_sheet_exporter::{BatchSheetContext, BatchSheetFormat};
use crate::application::session_guard::ensure_active_workspace;
use crate::domain::audit::audit_event::{Action, AuditEvent};

#[derive(Debug, Clone)]
pub struct ExportCartInput {
    pub format: BatchSheetFormat,
    pub out_path: PathBuf,
}

impl CartService {
    pub fn export_cart_as_batch_sheet(&self, input: ExportCartInput) -> AppResult<()> {
        let (_, workspace_id) = ensure_active_workspace(&*self.session_store)?;
        let lines = self.list_cart_with_calculations()?;

        // 只导出能算出结果的行；失败行交给 UI 提示用户先修配方。
        let results: Vec<_> = lines
            .into_iter()
            .filter_map(|l| l.calculation.ok())
            .collect();

        let workspace_name = self
            .workspaces_repo
            .find_by_id(workspace_id)?
            .map(|w| w.name().as_str().to_owned());
        let context = BatchSheetContext {
            workspace_name: workspace_name.as_deref(),
            ..Default::default()
        };

        self.batch_sheet_exporter
            .export(&results, context, input.format, &input.out_path)
            .map_err(|e| AppError::Io(e.to_string()))?;

        let format_str = match input.format {
            BatchSheetFormat::Csv => "csv",
            BatchSheetFormat::Html => "html",
            BatchSheetFormat::HtmlGrid => "html-grid",
        };
        let event = AuditEvent::new(
            Some(workspace_id),
            Action::CartExported,
            Some(input.out_path.to_string_lossy().into_owned()),
            Some(format!("format={format_str}")),
            self.clock.now(),
        );
        self.audit_writer.record(&event)?;
        Ok(())
    }
}
