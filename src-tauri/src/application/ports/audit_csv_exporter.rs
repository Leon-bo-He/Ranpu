use std::path::Path;

use crate::application::ports::batch_sheet_exporter::BatchSheetError;
use crate::domain::audit::audit_event::AuditEvent;

/// 把审计日志导出为明文 CSV（PROMPT 第 136 行 用户二次确认后才允许）。
///
/// 加密导出（.ydaexp）走 EncryptedExporter，与本 trait 互不重叠。
pub trait AuditCsvExporter: Send + Sync {
    fn export_csv(
        &self,
        events: &[AuditEvent],
        out_path: &Path,
    ) -> Result<(), BatchSheetError>;
}
