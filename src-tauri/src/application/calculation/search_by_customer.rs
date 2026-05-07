use crate::application::calculation::formula_resolver::CustomerCodeMatch;
use crate::application::calculation::service::CalculationService;
use crate::application::errors::AppResult;
use crate::application::session_guard::ensure_active_workspace;

#[derive(Debug, Clone)]
pub struct SearchByCustomerCodeInput {
    pub customer_color_code: String,
}

impl CalculationService {
    /// UI 入口：按客户色号查所有候选 (workspace + default), UI 让用户挑一条
    /// 之后再调 calculate_dye_amounts.
    ///
    /// 与 resolve_by_internal_code 不同, 这里不走 calc 也不写审计 —— 它只是搜索.
    pub fn search_candidates_by_customer_code(
        &self,
        input: SearchByCustomerCodeInput,
    ) -> AppResult<Vec<CustomerCodeMatch>> {
        let (_, workspace_id) = ensure_active_workspace(&*self.session_store)?;
        let trimmed = input.customer_color_code.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }
        self.search_by_customer_code(workspace_id, trimmed)
    }
}
