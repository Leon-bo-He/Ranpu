use crate::application::errors::AppResult;
use crate::application::formula::service::FormulaService;
use crate::application::ports::workspace_formula_repository::WorkspaceFormulaQuery;
use crate::application::session_guard::ensure_active_workspace;
use crate::domain::formula::workspace_formula::WorkspaceFormula;

#[derive(Debug, Clone)]
pub struct ListWorkspaceFormulasInput {
    pub keyword: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl FormulaService {
    pub fn list_workspace_formulas(
        &self,
        input: ListWorkspaceFormulasInput,
    ) -> AppResult<Vec<WorkspaceFormula>> {
        let (_, workspace_id) = ensure_active_workspace(&*self.session_store)?;
        let query = WorkspaceFormulaQuery {
            workspace_id,
            keyword: input.keyword.as_deref(),
            limit: input.limit,
            offset: input.offset,
        };
        Ok(self.workspace_repo.list(query)?)
    }
}
