use chrono::{DateTime, Utc};

use crate::application::audit::service::AuditService;
use crate::application::errors::AppResult;
use crate::application::ports::audit_repository::AuditQuery;
use crate::application::session_guard::ensure_active;
use crate::domain::audit::audit_event::{Action, AuditEvent};

#[derive(Debug, Clone, Default)]
pub struct ListAuditEventsInput {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub actions: Option<Vec<Action>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl AuditService {
    pub fn list_audit_events(
        &self,
        input: ListAuditEventsInput,
    ) -> AppResult<Vec<AuditEvent>> {
        let _ = ensure_active(&*self.session_store)?;
        let query = AuditQuery {
            from: input.from,
            to: input.to,
            actions: input.actions.as_deref(),
            limit: input.limit,
            offset: input.offset,
        };
        Ok(self.audit_repo.list(query)?)
    }
}
