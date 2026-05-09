use std::path::PathBuf;

use chrono::{DateTime, Utc};

use crate::application::audit::service::AuditService;
use crate::application::errors::{AppError, AppResult};
use crate::application::ports::audit_repository::AuditQuery;
use crate::application::session_guard::ensure_active;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::shared::errors::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditExportFormat {
    /// 加密 .ranpu（默认）。
    Encrypted,
    /// 明文 CSV。仅在 UI 二次确认后允许。
    PlainCsv,
}

#[derive(Debug, Clone)]
pub struct ExportAuditLogInput {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub actions: Option<Vec<Action>>,
    pub format: AuditExportFormat,
    pub passphrase: Option<String>,
    pub out_path: PathBuf,
}

impl AuditService {
    pub fn export_audit_log(&self, input: ExportAuditLogInput) -> AppResult<()> {
        let _ = ensure_active(&*self.session_store)?;
        if input.from > input.to {
            return Err(AppError::Domain(DomainError::AuditDateRangeInvalid));
        }

        let query = AuditQuery {
            from: Some(input.from),
            to: Some(input.to),
            actions: input.actions.as_deref(),
            limit: None,
            offset: None,
        };
        let events: Vec<AuditEvent> = self.audit_repo.list(query)?;

        match input.format {
            AuditExportFormat::PlainCsv => {
                self.csv_exporter
                    .export_csv(&events, &input.out_path)
                    .map_err(|e| AppError::Io(e.to_string()))?;
            }
            AuditExportFormat::Encrypted => {
                let passphrase = input
                    .passphrase
                    .as_ref()
                    .ok_or_else(|| AppError::Internal("加密导出需要口令".to_owned()))?;
                // 把审计事件序列化成 JSON 行（每事件一行），交给加密导出器。
                let mut buf = Vec::new();
                for e in &events {
                    use std::io::Write;
                    let line = format!(
                        "{{\"event_uuid\":\"{}\",\"workspace_id\":{},\"action\":\"{}\",\"target\":{},\"details\":{},\"occurred_at\":\"{}\"}}\n",
                        e.event_uuid(),
                        e.workspace_context_id().map(|i| i.to_string()).unwrap_or_else(|| "null".into()),
                        e.action().as_db_str(),
                        e.target().map(|s| format!("\"{}\"", s.replace('"', "\\\""))).unwrap_or_else(|| "null".into()),
                        e.details().map(|s| format!("\"{}\"", s.replace('"', "\\\""))).unwrap_or_else(|| "null".into()),
                        e.occurred_at().to_rfc3339(),
                    );
                    buf.write_all(line.as_bytes())
                        .map_err(|e| AppError::Io(e.to_string()))?;
                }
                self.encrypted_exporter
                    .export_to_file(&buf, passphrase, &input.out_path)
                    .map_err(|e| AppError::Crypto(e.to_string()))?;
            }
        }

        let format_str = match input.format {
            AuditExportFormat::Encrypted => "encrypted",
            AuditExportFormat::PlainCsv => "csv",
        };
        let event = AuditEvent::new(
            None,
            Action::AuditLogExported,
            Some(input.out_path.to_string_lossy().into_owned()),
            Some(format!(
                "format={};count={};from={};to={}",
                format_str,
                events.len(),
                input.from.to_rfc3339(),
                input.to.to_rfc3339()
            )),
            self.clock.now(),
        );
        self.audit_writer.record(&event)?;
        Ok(())
    }
}
