use chrono::{DateTime, Utc};

use crate::application::ports::errors::RepositoryError;
use crate::domain::audit::audit_event::{Action, AuditEvent};
use crate::domain::shared::id::{AuditEventId, UserId};

#[derive(Debug, Clone)]
pub struct AuditQuery<'a> {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub user_ids: Option<&'a [UserId]>,
    pub actions: Option<&'a [Action]>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

pub trait AuditRepository: Send + Sync {
    fn list(&self, query: AuditQuery<'_>) -> Result<Vec<AuditEvent>, RepositoryError>;
    fn find_by_id(&self, id: AuditEventId) -> Result<Option<AuditEvent>, RepositoryError>;
}
