use std::path::Path;

use crate::application::ports::audit_csv_exporter::AuditCsvExporter;
use crate::application::ports::batch_sheet_exporter::BatchSheetError;
use crate::domain::audit::audit_event::AuditEvent;

pub struct PlainAuditCsvExporter;

impl PlainAuditCsvExporter {
    pub const fn new() -> Self {
        Self
    }
}

impl Default for PlainAuditCsvExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditCsvExporter for PlainAuditCsvExporter {
    fn export_csv(
        &self,
        events: &[AuditEvent],
        out_path: &Path,
    ) -> Result<(), BatchSheetError> {
        let mut buf = String::new();
        buf.push_str("时间,事件 UUID,工作区 ID,动作,目标,详情\n");
        for e in events {
            buf.push_str(&format!(
                "{},{},{},{},{},{}\n",
                e.occurred_at().format("%Y-%m-%d %H:%M:%S"),
                e.event_uuid(),
                e.workspace_context_id()
                    .map(|i| i.to_string())
                    .unwrap_or_default(),
                e.action().as_db_str(),
                csv_escape(e.target().unwrap_or("")),
                csv_escape(e.details().unwrap_or("")),
            ));
        }
        std::fs::write(out_path, buf).map_err(|e| BatchSheetError::Io(e.to_string()))?;
        Ok(())
    }
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_owned()
    }
}
