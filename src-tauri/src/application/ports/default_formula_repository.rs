use crate::application::ports::errors::RepositoryError;
use crate::domain::formula::default_formula::DefaultFormula;
use crate::domain::formula::internal_color_code::InternalColorCode;
use crate::domain::shared::id::FormulaId;

#[derive(Debug, Clone)]
pub struct DefaultFormulaQuery<'a> {
    /// 模糊关键词：同时匹配 内部色号 / 客户色号 / 颜色俗称（PROMPT 第 218 行）。
    pub keyword: Option<&'a str>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

pub trait DefaultFormulaRepository: Send + Sync {
    fn find_by_id(&self, id: FormulaId) -> Result<Option<DefaultFormula>, RepositoryError>;
    fn find_by_internal_code(
        &self,
        code: &InternalColorCode,
    ) -> Result<Option<DefaultFormula>, RepositoryError>;
    fn find_by_customer_code(
        &self,
        customer_code: &str,
    ) -> Result<Vec<DefaultFormula>, RepositoryError>;
    fn list(&self, query: DefaultFormulaQuery<'_>) -> Result<Vec<DefaultFormula>, RepositoryError>;
    /// id 为 None → 插入；为 Some → 整体覆盖（含 items）。单事务。
    fn upsert(&self, formula: &DefaultFormula) -> Result<FormulaId, RepositoryError>;
    fn delete(&self, id: FormulaId) -> Result<(), RepositoryError>;
}
