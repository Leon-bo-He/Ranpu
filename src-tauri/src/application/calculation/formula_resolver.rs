//! 配方解析：同一内部色号先查激活 workspace，找不到再 fallback 到 default 库。
//! 客户色号查询则返回多匹配，由用户选择具体一条（PROMPT 第 122 行）。

use crate::application::calculation::service::CalculationService;
use crate::application::errors::{AppError, AppResult};
use crate::application::ports::errors::RepositoryError;
use crate::domain::calculation::dye_calculator::{CalculableFormula, FormulaSource};
use crate::domain::formula::default_formula::DefaultFormula;
use crate::domain::formula::internal_color_code::InternalColorCode;
use crate::domain::formula::workspace_formula::WorkspaceFormula;
use crate::domain::shared::id::{FormulaId, WorkspaceId};

/// 解析后的配方包装。两种聚合都要能喂进 DyeCalculator。
#[derive(Debug, Clone)]
pub enum ResolvedFormula {
    Workspace(WorkspaceFormula),
    Default(DefaultFormula),
}

impl ResolvedFormula {
    pub fn source(&self) -> FormulaSource {
        match self {
            ResolvedFormula::Workspace(_) => FormulaSource::CurrentWorkspace,
            ResolvedFormula::Default(_) => FormulaSource::DefaultFallback,
        }
    }

    pub fn as_calculable(&self) -> &dyn CalculableFormula {
        match self {
            ResolvedFormula::Workspace(f) => f,
            ResolvedFormula::Default(f) => f,
        }
    }
}

/// 客户色号查询结果（可能多条，UI 让用户挑）。
#[derive(Debug, Clone)]
pub struct CustomerCodeMatch {
    pub source: FormulaSource,
    pub formula_id: Option<FormulaId>,
    pub internal_color_code: InternalColorCode,
    pub color_family: Option<String>,
    pub customer_color_code: Option<String>,
}

impl CalculationService {
    /// 按内部色号解析：先 workspace 后 default 库。
    pub fn resolve_by_internal_code(
        &self,
        workspace_id: WorkspaceId,
        code: &InternalColorCode,
    ) -> AppResult<ResolvedFormula> {
        if let Some(f) = self
            .workspace_repo
            .find_by_internal_code(workspace_id, code)?
        {
            return Ok(ResolvedFormula::Workspace(f));
        }
        if let Some(f) = self.default_repo.find_by_internal_code(code)? {
            return Ok(ResolvedFormula::Default(f));
        }
        Err(AppError::Repository(RepositoryError::NotFound))
    }

    /// 按客户色号查询所有候选（workspace + default 都查），UI 让用户挑一条，
    /// 拿到内部色号后再走 resolve_by_internal_code → calculate_dye_amounts。
    pub fn search_by_customer_code(
        &self,
        workspace_id: WorkspaceId,
        customer_code: &str,
    ) -> AppResult<Vec<CustomerCodeMatch>> {
        let mut out = Vec::new();
        for f in self
            .workspace_repo
            .find_by_customer_code(workspace_id, customer_code)?
        {
            out.push(CustomerCodeMatch {
                source: FormulaSource::CurrentWorkspace,
                formula_id: f.id(),
                internal_color_code: <WorkspaceFormula as CalculableFormula>::internal_color_code(&f).clone(),
                color_family: f.color_family().map(str::to_owned),
                customer_color_code: f.customer_color_code().map(|c| c.as_str().to_owned()),
            });
        }
        for f in self.default_repo.find_by_customer_code(customer_code)? {
            out.push(CustomerCodeMatch {
                source: FormulaSource::DefaultFallback,
                formula_id: f.id(),
                internal_color_code: <DefaultFormula as CalculableFormula>::internal_color_code(&f).clone(),
                color_family: f.color_family().map(str::to_owned),
                customer_color_code: f.customer_color_code().map(|c| c.as_str().to_owned()),
            });
        }
        Ok(out)
    }
}
