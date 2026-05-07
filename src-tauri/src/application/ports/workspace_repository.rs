use crate::application::ports::errors::RepositoryError;
use crate::domain::shared::id::WorkspaceId;
use crate::domain::workspace::workspace::Workspace;

pub trait WorkspaceRepository: Send + Sync {
    fn find_by_id(&self, id: WorkspaceId) -> Result<Option<Workspace>, RepositoryError>;
    fn find_by_name(&self, name: &str) -> Result<Option<Workspace>, RepositoryError>;
    fn list_all(&self) -> Result<Vec<Workspace>, RepositoryError>;
    fn insert(&self, workspace: &Workspace) -> Result<WorkspaceId, RepositoryError>;
    fn rename(&self, id: WorkspaceId, new_name: &str) -> Result<(), RepositoryError>;
    fn update_description(
        &self,
        id: WorkspaceId,
        description: Option<&str>,
    ) -> Result<(), RepositoryError>;
    fn delete(&self, id: WorkspaceId) -> Result<(), RepositoryError>;
}
