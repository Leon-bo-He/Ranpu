use crate::application::cart::service::CartService;
use crate::application::errors::{AppError, AppResult};
use crate::application::ports::errors::RepositoryError;
use crate::application::session_guard::ensure_active_workspace;
use crate::domain::calculation::dye_calculator::{
    CalculableFormula, CalculationResult, FormulaSource,
};
use crate::domain::cart::cart_item::{CartItem, SourceKind};

/// 一个购物车条目 + 其计算结果（或解析失败的原因）。
#[derive(Debug, Clone)]
pub struct CartLine {
    pub item: CartItem,
    /// 显示给 UI 的内部色号（解析后的）。
    pub internal_color_code: Option<String>,
    pub color_name: Option<String>,
    pub customer_color_code: Option<String>,
    /// 计算结果。配方被删/不见了时是 NotFound，其它领域错按原文返回。
    pub calculation: Result<CalculationResult, String>,
}

impl CartService {
    pub fn list_cart_with_calculations(&self) -> AppResult<Vec<CartLine>> {
        let (snap, workspace_id) = ensure_active_workspace(&*self.session_store)?;
        let cart = self.cart_repo.load(snap.user_id(), workspace_id)?;
        let mut out = Vec::with_capacity(cart.items().len());

        for item in cart.items() {
            let line = self.compute_line(workspace_id, item.clone());
            out.push(line);
        }
        Ok(out)
    }

    fn compute_line(
        &self,
        workspace_id: crate::domain::shared::id::WorkspaceId,
        item: CartItem,
    ) -> CartLine {
        // 把可能失败的步骤拆开，捕捉到 String 错误返回给 UI。
        match item.source_kind() {
            SourceKind::Workspace => match self
                .workspace_repo
                .find_by_id(workspace_id, item.source_formula_id())
            {
                Ok(Some(f)) => {
                    let calculable: &dyn CalculableFormula = &f;
                    let result = self.calculator.calculate(
                        calculable,
                        item.target_kg(),
                        FormulaSource::CurrentWorkspace,
                    );
                    CartLine {
                        internal_color_code: Some(
                            <crate::domain::formula::workspace_formula::WorkspaceFormula as CalculableFormula>::internal_color_code(&f)
                                .as_str()
                                .to_owned(),
                        ),
                        color_name: f.color_name().map(str::to_owned),
                        customer_color_code: f
                            .customer_color_code()
                            .map(|c| c.as_str().to_owned()),
                        calculation: result.map_err(|e| e.to_string()),
                        item,
                    }
                }
                Ok(None) => CartLine {
                    item,
                    internal_color_code: None,
                    color_name: None,
                    customer_color_code: None,
                    calculation: Err(format!("{}", AppError::Repository(RepositoryError::NotFound))),
                },
                Err(e) => CartLine {
                    item,
                    internal_color_code: None,
                    color_name: None,
                    customer_color_code: None,
                    calculation: Err(e.to_string()),
                },
            },
            SourceKind::Default => match self.default_repo.find_by_id(item.source_formula_id()) {
                Ok(Some(f)) => {
                    let calculable: &dyn CalculableFormula = &f;
                    let result = self.calculator.calculate(
                        calculable,
                        item.target_kg(),
                        FormulaSource::DefaultFallback,
                    );
                    CartLine {
                        internal_color_code: Some(
                            <crate::domain::formula::default_formula::DefaultFormula as CalculableFormula>::internal_color_code(&f)
                                .as_str()
                                .to_owned(),
                        ),
                        color_name: f.color_name().map(str::to_owned),
                        customer_color_code: f
                            .customer_color_code()
                            .map(|c| c.as_str().to_owned()),
                        calculation: result.map_err(|e| e.to_string()),
                        item,
                    }
                }
                Ok(None) => CartLine {
                    item,
                    internal_color_code: None,
                    color_name: None,
                    customer_color_code: None,
                    calculation: Err(format!("{}", AppError::Repository(RepositoryError::NotFound))),
                },
                Err(e) => CartLine {
                    item,
                    internal_color_code: None,
                    color_name: None,
                    customer_color_code: None,
                    calculation: Err(e.to_string()),
                },
            },
        }
    }
}
