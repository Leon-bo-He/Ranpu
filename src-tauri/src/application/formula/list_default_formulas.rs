use crate::application::errors::AppResult;
use crate::application::formula::service::FormulaService;
use crate::application::ports::default_formula_repository::DefaultFormulaQuery;
use crate::application::session_guard::ensure_active;
use crate::domain::formula::default_formula::DefaultFormula;

#[derive(Debug, Clone)]
pub struct ListDefaultFormulasInput {
    pub keyword: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl FormulaService {
    pub fn list_default_formulas(
        &self,
        input: ListDefaultFormulasInput,
    ) -> AppResult<Vec<DefaultFormula>> {
        let _ = ensure_active(&*self.session_store)?;
        let query = DefaultFormulaQuery {
            keyword: input.keyword.as_deref(),
            limit: input.limit,
            offset: input.offset,
        };
        Ok(self.default_repo.list(query)?)
    }
}
