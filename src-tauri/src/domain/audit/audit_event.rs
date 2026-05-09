use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::shared::errors::DomainError;
use crate::domain::shared::id::{AuditEventId, WorkspaceId};

/// 审计动作类型。
///
/// 单用户解锁模型: login / account / password / user_* 这些事件没有了 —
/// 没有用户体系自然就写不出这些. 保留 SessionLocked / SessionUnlocked
/// (锁屏 / 解锁).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    SessionLocked,
    SessionUnlocked,
    WorkspaceCreated,
    WorkspaceRenamed,
    WorkspaceDescriptionUpdated,
    WorkspaceDeleted,
    WorkspaceSwitched,
    DefaultFormulaUpserted,
    DefaultFormulaDeleted,
    WorkspaceFormulaUpserted,
    WorkspaceFormulaDeleted,
    DefaultFormulaCopiedToWorkspace,
    DefaultFormulasExported,
    DefaultFormulasImported,
    WorkspaceFormulasExported,
    WorkspaceFormulasImported,
    LibraryArchiveExported,
    LibraryArchiveImported,
    CalculationPerformed,
    CartItemAdded,
    CartItemRemoved,
    CartItemKgUpdated,
    CartCleared,
    CartExported,
    BackupExported,
    BackupImported,
    AuditLogExported,
}

impl Action {
    pub const fn as_db_str(self) -> &'static str {
        match self {
            Action::SessionLocked => "session_locked",
            Action::SessionUnlocked => "session_unlocked",
            Action::WorkspaceCreated => "workspace_created",
            Action::WorkspaceRenamed => "workspace_renamed",
            Action::WorkspaceDescriptionUpdated => "workspace_description_updated",
            Action::WorkspaceDeleted => "workspace_deleted",
            Action::WorkspaceSwitched => "workspace_switched",
            Action::DefaultFormulaUpserted => "default_formula_upserted",
            Action::DefaultFormulaDeleted => "default_formula_deleted",
            Action::WorkspaceFormulaUpserted => "workspace_formula_upserted",
            Action::WorkspaceFormulaDeleted => "workspace_formula_deleted",
            Action::DefaultFormulaCopiedToWorkspace => "default_formula_copied_to_workspace",
            Action::DefaultFormulasExported => "default_formulas_exported",
            Action::DefaultFormulasImported => "default_formulas_imported",
            Action::WorkspaceFormulasExported => "workspace_formulas_exported",
            Action::WorkspaceFormulasImported => "workspace_formulas_imported",
            Action::LibraryArchiveExported => "library_archive_exported",
            Action::LibraryArchiveImported => "library_archive_imported",
            Action::CalculationPerformed => "calculation_performed",
            Action::CartItemAdded => "cart_item_added",
            Action::CartItemRemoved => "cart_item_removed",
            Action::CartItemKgUpdated => "cart_item_kg_updated",
            Action::CartCleared => "cart_cleared",
            Action::CartExported => "cart_exported",
            Action::BackupExported => "backup_exported",
            Action::BackupImported => "backup_imported",
            Action::AuditLogExported => "audit_log_exported",
        }
    }
}

impl FromStr for Action {
    type Err = DomainError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // 将 db_str 反查回枚举值，避免维护两份字符串映射。
        const ALL: &[Action] = &[
            Action::SessionLocked,
            Action::SessionUnlocked,
            Action::WorkspaceCreated,
            Action::WorkspaceRenamed,
            Action::WorkspaceDescriptionUpdated,
            Action::WorkspaceDeleted,
            Action::WorkspaceSwitched,
            Action::DefaultFormulaUpserted,
            Action::DefaultFormulaDeleted,
            Action::WorkspaceFormulaUpserted,
            Action::WorkspaceFormulaDeleted,
            Action::DefaultFormulaCopiedToWorkspace,
            Action::DefaultFormulasExported,
            Action::DefaultFormulasImported,
            Action::WorkspaceFormulasExported,
            Action::WorkspaceFormulasImported,
            Action::LibraryArchiveExported,
            Action::LibraryArchiveImported,
            Action::CalculationPerformed,
            Action::CartItemAdded,
            Action::CartItemRemoved,
            Action::CartItemKgUpdated,
            Action::CartCleared,
            Action::CartExported,
            Action::BackupExported,
            Action::BackupImported,
            Action::AuditLogExported,
        ];
        ALL.iter()
            .copied()
            .find(|a| a.as_db_str() == s)
            .ok_or_else(|| DomainError::UnknownUnit(s.to_owned()))
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_db_str())
    }
}

/// 审计事件实体。
///
/// 一条事件由 application 层在用例完成时构造（含失败用例），写入仓储。
/// `target` 与 `details` 可空：target 是被操作对象的标识（比如配方内部色号
/// 或工作区 id），details 是结构化信息（一般是 JSON 字符串）。
///
/// 单用户解锁模型: 没有 user_id 字段 — 操作主体只可能是当前解锁的人.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEvent {
    id: Option<AuditEventId>,
    /// 服务端生成的 UUID，给离线审计工具一个稳定句柄。
    event_uuid: Uuid,
    workspace_context_id: Option<WorkspaceId>,
    action: Action,
    target: Option<String>,
    details: Option<String>,
    occurred_at: DateTime<Utc>,
}

impl AuditEvent {
    pub fn new(
        workspace_context_id: Option<WorkspaceId>,
        action: Action,
        target: Option<String>,
        details: Option<String>,
        occurred_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: None,
            event_uuid: Uuid::new_v4(),
            workspace_context_id,
            action,
            target,
            details,
            occurred_at,
        }
    }

    pub fn rehydrate(
        id: AuditEventId,
        event_uuid: Uuid,
        workspace_context_id: Option<WorkspaceId>,
        action: Action,
        target: Option<String>,
        details: Option<String>,
        occurred_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            event_uuid,
            workspace_context_id,
            action,
            target,
            details,
            occurred_at,
        }
    }

    pub fn id(&self) -> Option<AuditEventId> {
        self.id
    }
    pub fn event_uuid(&self) -> Uuid {
        self.event_uuid
    }
    pub fn workspace_context_id(&self) -> Option<WorkspaceId> {
        self.workspace_context_id
    }
    pub fn action(&self) -> Action {
        self.action
    }
    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }
    pub fn details(&self) -> Option<&str> {
        self.details.as_deref()
    }
    pub fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    pub fn assign_id(&mut self, id: AuditEventId) {
        self.id = Some(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn t() -> DateTime<Utc> {
        Utc.timestamp_opt(0, 0).unwrap()
    }

    #[test]
    fn action_round_trips_through_db_str() {
        for a in [
            Action::SessionLocked,
            Action::CartItemAdded,
            Action::WorkspaceFormulaUpserted,
            Action::AuditLogExported,
        ] {
            assert_eq!(Action::from_str(a.as_db_str()).unwrap(), a);
        }
    }

    #[test]
    fn unknown_action_string_is_rejected() {
        assert!(Action::from_str("nope").is_err());
    }

    #[test]
    fn new_event_has_no_id_but_has_uuid() {
        let e = AuditEvent::new(
            Some(WorkspaceId::new(2)),
            Action::SessionUnlocked,
            Some("alice".into()),
            None,
            t(),
        );
        assert!(e.id().is_none());
        assert_ne!(e.event_uuid(), Uuid::nil());
        assert_eq!(e.action(), Action::SessionUnlocked);
        assert_eq!(e.target(), Some("alice"));
    }
}
