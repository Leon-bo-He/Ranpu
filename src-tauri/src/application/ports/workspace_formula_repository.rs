use crate::application::ports::errors::RepositoryError;
use crate::domain::formula::default_formula::DefaultFormula;
use crate::domain::formula::internal_color_code::InternalColorCode;
use crate::domain::formula::workspace_formula::WorkspaceFormula;
use crate::domain::shared::id::{FormulaId, WorkspaceId};

#[derive(Debug, Clone)]
pub struct WorkspaceFormulaQuery<'a> {
    pub workspace_id: WorkspaceId,
    pub keyword: Option<&'a str>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

pub trait WorkspaceFormulaRepository: Send + Sync {
    fn find_by_id(
        &self,
        workspace_id: WorkspaceId,
        id: FormulaId,
    ) -> Result<Option<WorkspaceFormula>, RepositoryError>;

    fn find_by_internal_code(
        &self,
        workspace_id: WorkspaceId,
        code: &InternalColorCode,
    ) -> Result<Option<WorkspaceFormula>, RepositoryError>;

    fn find_by_customer_code(
        &self,
        workspace_id: WorkspaceId,
        customer_code: &str,
    ) -> Result<Vec<WorkspaceFormula>, RepositoryError>;

    fn list(
        &self,
        query: WorkspaceFormulaQuery<'_>,
    ) -> Result<Vec<WorkspaceFormula>, RepositoryError>;

    /// 列出该工作区已用过的色系 (distinct, 字典序), 给前端 dropdown 用.
    fn list_color_families(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Vec<String>, RepositoryError>;

    fn upsert(&self, formula: &WorkspaceFormula) -> Result<FormulaId, RepositoryError>;

    fn delete(&self, workspace_id: WorkspaceId, id: FormulaId) -> Result<(), RepositoryError>;

    fn copy_from_default(
        &self,
        default: &DefaultFormula,
        workspace_id: WorkspaceId,
    ) -> Result<FormulaId, RepositoryError>;
}
