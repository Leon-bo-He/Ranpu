use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};

use crate::domain::shared::errors::{DomainError, DomainResult};
use crate::domain::shared::id::WorkspaceId;

/// 工作区类型。
///
/// - `Normal`: 用户创建的常规工作区, 配方可自由增删改.
/// - `SystemMirror`: 系统内置 "通用" 工作区, 配方与默认配方库一一同步,
///   不可在此工作区内直接增删改, 工作区本身也不能改名 / 删除.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkspaceKind {
    Normal,
    SystemMirror,
}

impl WorkspaceKind {
    pub const fn as_db_str(self) -> &'static str {
        match self {
            WorkspaceKind::Normal => "normal",
            WorkspaceKind::SystemMirror => "system_mirror",
        }
    }
}

impl FromStr for WorkspaceKind {
    type Err = DomainError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "normal" => Ok(WorkspaceKind::Normal),
            "system_mirror" => Ok(WorkspaceKind::SystemMirror),
            other => Err(DomainError::UnknownUnit(other.to_owned())),
        }
    }
}

/// 工作区名称值对象：1-64 字符。
///
/// 唯一性由仓储/数据库索引保障。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkspaceName(String);

impl WorkspaceName {
    pub fn new(raw: impl Into<String>) -> DomainResult<Self> {
        let s = raw.into();
        let trimmed = s.trim();
        let len = trimmed.chars().count();
        if len == 0 {
            return Err(DomainError::WorkspaceNameEmpty);
        }
        if len > 64 {
            return Err(DomainError::WorkspaceNameTooLong { len });
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WorkspaceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Workspace 聚合根。
///
/// 一个独立的车间工作站工作区，归属一个客户/项目。
/// 单用户解锁模型: 没有 created_by_user_id (没有用户表).
#[derive(Debug, Clone, PartialEq)]
pub struct Workspace {
    id: Option<WorkspaceId>,
    name: WorkspaceName,
    description: Option<String>,
    created_at: DateTime<Utc>,
    kind: WorkspaceKind,
}

impl Workspace {
    /// 构造一个尚未持久化的 Workspace（默认 `Normal` 类型）。
    pub fn new(
        name: WorkspaceName,
        description: Option<String>,
        created_at: DateTime<Utc>,
    ) -> DomainResult<Self> {
        Self::new_with_kind(name, description, created_at, WorkspaceKind::Normal)
    }

    pub fn new_with_kind(
        name: WorkspaceName,
        description: Option<String>,
        created_at: DateTime<Utc>,
        kind: WorkspaceKind,
    ) -> DomainResult<Self> {
        let description = normalize_description(description)?;
        Ok(Self {
            id: None,
            name,
            description,
            created_at,
            kind,
        })
    }

    pub fn rehydrate(
        id: WorkspaceId,
        name: WorkspaceName,
        description: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self::rehydrate_with_kind(id, name, description, created_at, WorkspaceKind::Normal)
    }

    pub fn rehydrate_with_kind(
        id: WorkspaceId,
        name: WorkspaceName,
        description: Option<String>,
        created_at: DateTime<Utc>,
        kind: WorkspaceKind,
    ) -> Self {
        Self {
            id: Some(id),
            name,
            description,
            created_at,
            kind,
        }
    }

    pub fn id(&self) -> Option<WorkspaceId> {
        self.id
    }

    pub fn name(&self) -> &WorkspaceName {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn kind(&self) -> WorkspaceKind {
        self.kind
    }

    pub fn is_system_mirror(&self) -> bool {
        self.kind == WorkspaceKind::SystemMirror
    }

    pub fn assign_id(&mut self, id: WorkspaceId) {
        self.id = Some(id);
    }

    pub fn rename(&mut self, new_name: WorkspaceName) {
        self.name = new_name;
    }

    pub fn set_description(&mut self, description: Option<String>) -> DomainResult<()> {
        self.description = normalize_description(description)?;
        Ok(())
    }
}

fn normalize_description(d: Option<String>) -> DomainResult<Option<String>> {
    match d {
        None => Ok(None),
        Some(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            let len = trimmed.chars().count();
            if len > 1024 {
                return Err(DomainError::DescriptionTooLong { len });
            }
            Ok(Some(trimmed.to_owned()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn now() -> DateTime<Utc> {
        Utc.timestamp_opt(0, 0).unwrap()
    }

    #[test]
    fn workspace_name_trims_and_validates() {
        let n = WorkspaceName::new("  客户A  ").unwrap();
        assert_eq!(n.as_str(), "客户A");
    }

    #[test]
    fn workspace_name_rejects_blank() {
        assert!(matches!(
            WorkspaceName::new("   "),
            Err(DomainError::WorkspaceNameEmpty)
        ));
    }

    #[test]
    fn workspace_name_rejects_too_long() {
        let s = "字".repeat(65);
        assert!(matches!(
            WorkspaceName::new(s),
            Err(DomainError::WorkspaceNameTooLong { len: 65 })
        ));
    }

    #[test]
    fn new_workspace_has_no_id() {
        let w = Workspace::new(WorkspaceName::new("X").unwrap(), None, now()).unwrap();
        assert!(w.id().is_none());
    }

    #[test]
    fn description_blank_normalizes_to_none() {
        let w = Workspace::new(
            WorkspaceName::new("X").unwrap(),
            Some("   ".into()),
            now(),
        )
        .unwrap();
        assert!(w.description().is_none());
    }

    #[test]
    fn description_too_long_is_rejected() {
        let too_long = "字".repeat(1025);
        let err = Workspace::new(
            WorkspaceName::new("X").unwrap(),
            Some(too_long),
            now(),
        )
        .unwrap_err();
        assert!(matches!(err, DomainError::DescriptionTooLong { .. }));
    }

    #[test]
    fn rename_updates_name() {
        let mut w = Workspace::new(WorkspaceName::new("旧名").unwrap(), None, now()).unwrap();
        w.rename(WorkspaceName::new("新名").unwrap());
        assert_eq!(w.name().as_str(), "新名");
    }
}
