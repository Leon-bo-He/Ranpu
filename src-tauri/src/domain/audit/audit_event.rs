use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::shared::errors::DomainError;
use crate::domain::shared::id::{AuditEventId, UserId, WorkspaceId};

/// 审计动作类型。
///
/// 列表对应 PROMPT 各上下文用例 + 生命周期事件。导出审计日志时按这里
/// 的字符串表示存盘，便于离线审计工具按字符串匹配。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    LoginSucceeded,
    LoginFailed,
    AccountLocked,
    AccountUnlocked,
    SessionLocked,
    SessionUnlocked,
    SessionForceLogout,
    PasswordChanged,
    UserCreated,
    UserDeactivated,
    WorkspaceCreated,
    WorkspaceRenamed,
    WorkspaceDeleted,
    WorkspaceSwitched,
    DefaultFormulaUpserted,
    DefaultFormulaDeleted,
    WorkspaceFormulaUpserted,
    WorkspaceFormulaDeleted,
    DefaultFormulaCopiedToWorkspace,
    DefaultFormulasExported,
    DefaultFormulasImported,
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
            Action::LoginSucceeded => "login_succeeded",
            Action::LoginFailed => "login_failed",
            Action::AccountLocked => "account_locked",
            Action::AccountUnlocked => "account_unlocked",
            Action::SessionLocked => "session_locked",
            Action::SessionUnlocked => "session_unlocked",
            Action::SessionForceLogout => "session_force_logout",
            Action::PasswordChanged => "password_changed",
            Action::UserCreated => "user_created",
            Action::UserDeactivated => "user_deactivated",
            Action::WorkspaceCreated => "workspace_created",
            Action::WorkspaceRenamed => "workspace_renamed",
            Action::WorkspaceDeleted => "workspace_deleted",
            Action::WorkspaceSwitched => "workspace_switched",
            Action::DefaultFormulaUpserted => "default_formula_upserted",
            Action::DefaultFormulaDeleted => "default_formula_deleted",
            Action::WorkspaceFormulaUpserted => "workspace_formula_upserted",
            Action::WorkspaceFormulaDeleted => "workspace_formula_deleted",
            Action::DefaultFormulaCopiedToWorkspace => "default_formula_copied_to_workspace",
            Action::DefaultFormulasExported => "default_formulas_exported",
            Action::DefaultFormulasImported => "default_formulas_imported",
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
            Action::LoginSucceeded,
            Action::LoginFailed,
            Action::AccountLocked,
            Action::AccountUnlocked,
            Action::SessionLocked,
            Action::SessionUnlocked,
            Action::SessionForceLogout,
            Action::PasswordChanged,
            Action::UserCreated,
            Action::UserDeactivated,
            Action::WorkspaceCreated,
            Action::WorkspaceRenamed,
            Action::WorkspaceDeleted,
            Action::WorkspaceSwitched,
            Action::DefaultFormulaUpserted,
            Action::DefaultFormulaDeleted,
            Action::WorkspaceFormulaUpserted,
            Action::WorkspaceFormulaDeleted,
            Action::DefaultFormulaCopiedToWorkspace,
            Action::DefaultFormulasExported,
            Action::DefaultFormulasImported,
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
/// 或 user_id），details 是结构化信息（一般是 JSON 字符串）。
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEvent {
    id: Option<AuditEventId>,
    /// 服务端生成的 UUID，给离线审计工具一个稳定句柄。
    event_uuid: Uuid,
    user_id: Option<UserId>,
    workspace_context_id: Option<WorkspaceId>,
    action: Action,
    target: Option<String>,
    details: Option<String>,
    occurred_at: DateTime<Utc>,
}

impl AuditEvent {
    pub fn new(
        user_id: Option<UserId>,
        workspace_context_id: Option<WorkspaceId>,
        action: Action,
        target: Option<String>,
        details: Option<String>,
        occurred_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: None,
            event_uuid: Uuid::new_v4(),
            user_id,
            workspace_context_id,
            action,
            target,
            details,
            occurred_at,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: AuditEventId,
        event_uuid: Uuid,
        user_id: Option<UserId>,
        workspace_context_id: Option<WorkspaceId>,
        action: Action,
        target: Option<String>,
        details: Option<String>,
        occurred_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            event_uuid,
            user_id,
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
    pub fn user_id(&self) -> Option<UserId> {
        self.user_id
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
            Action::LoginSucceeded,
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
            Some(UserId::new(1)),
            Some(WorkspaceId::new(2)),
            Action::LoginSucceeded,
            Some("alice".into()),
            None,
            t(),
        );
        assert!(e.id().is_none());
        assert_ne!(e.event_uuid(), Uuid::nil());
        assert_eq!(e.action(), Action::LoginSucceeded);
        assert_eq!(e.target(), Some("alice"));
    }
}
