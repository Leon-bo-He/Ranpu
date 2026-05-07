use std::fmt;

use chrono::{DateTime, Utc};

use crate::domain::shared::errors::{DomainError, DomainResult};
use crate::domain::shared::id::{UserId, WorkspaceId};

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
/// 任何登录用户都可以切换进入；只有 admin 能创建/重命名/删除（PROMPT 第 63 行）。
#[derive(Debug, Clone, PartialEq)]
pub struct Workspace {
    id: Option<WorkspaceId>,
    name: WorkspaceName,
    description: Option<String>,
    created_by_user_id: Option<UserId>,
    created_at: DateTime<Utc>,
}

impl Workspace {
    /// 构造一个尚未持久化的 Workspace。
    /// `created_by_user_id` 允许 None：seed/系统初始化时没有用户。
    pub fn new(
        name: WorkspaceName,
        description: Option<String>,
        created_by_user_id: Option<UserId>,
        created_at: DateTime<Utc>,
    ) -> DomainResult<Self> {
        let description = normalize_description(description)?;
        Ok(Self {
            id: None,
            name,
            description,
            created_by_user_id,
            created_at,
        })
    }

    pub fn rehydrate(
        id: WorkspaceId,
        name: WorkspaceName,
        description: Option<String>,
        created_by_user_id: Option<UserId>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            name,
            description,
            created_by_user_id,
            created_at,
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

    pub fn created_by_user_id(&self) -> Option<UserId> {
        self.created_by_user_id
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
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
        let w = Workspace::new(
            WorkspaceName::new("X").unwrap(),
            None,
            Some(UserId::new(1)),
            now(),
        )
        .unwrap();
        assert!(w.id().is_none());
        assert_eq!(w.created_by_user_id(), Some(UserId::new(1)));
    }

    #[test]
    fn description_blank_normalizes_to_none() {
        let w = Workspace::new(
            WorkspaceName::new("X").unwrap(),
            Some("   ".into()),
            Some(UserId::new(1)),
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
            Some(UserId::new(1)),
            now(),
        )
        .unwrap_err();
        assert!(matches!(err, DomainError::DescriptionTooLong { .. }));
    }

    #[test]
    fn rename_updates_name() {
        let mut w = Workspace::new(
            WorkspaceName::new("旧名").unwrap(),
            None,
            Some(UserId::new(1)),
            now(),
        )
        .unwrap();
        w.rename(WorkspaceName::new("新名").unwrap());
        assert_eq!(w.name().as_str(), "新名");
    }
}
