use crate::application::errors::AppResult;
use crate::application::formula::service::FormulaService;
use crate::application::session_guard::ensure_active_workspace;
use crate::domain::shared::id::FormulaId;

#[derive(Debug, Clone)]
pub struct BatchCopyDefaultInput {
    pub default_formula_ids: Vec<FormulaId>,
}

#[derive(Debug, Clone)]
pub struct BatchCopyOutcomeItem {
    pub source_default_id: FormulaId,
    pub new_workspace_formula_id: Option<FormulaId>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BatchCopySummary {
    pub items: Vec<BatchCopyOutcomeItem>,
    pub succeeded: u32,
    pub failed: u32,
}

impl FormulaService {
    /// 批量复制默认库配方到当前工作区。
    ///
    /// 单条失败不中断后续；返回逐项结果 + 总计。每条成功的复制还是会
    /// 单独写一笔 `DefaultFormulaCopiedToWorkspace` 审计 (复用现有
    /// copy_default_to_active_workspace 路径)。
    pub fn batch_copy_default_to_active_workspace(
        &self,
        input: BatchCopyDefaultInput,
    ) -> AppResult<BatchCopySummary> {
        // 上来快速失败：未解锁 / 未激活 workspace / 系统镜像 都不需要进循环。
        let (_, workspace_id) = ensure_active_workspace(&*self.session_store)?;
        self.reject_if_system_mirror(workspace_id)?;

        let mut items = Vec::with_capacity(input.default_formula_ids.len());
        let mut succeeded = 0_u32;
        let mut failed = 0_u32;
        for id in input.default_formula_ids {
            match self.copy_default_to_active_workspace(id) {
                Ok(new_id) => {
                    succeeded += 1;
                    items.push(BatchCopyOutcomeItem {
                        source_default_id: id,
                        new_workspace_formula_id: Some(new_id),
                        error: None,
                    });
                }
                Err(e) => {
                    failed += 1;
                    items.push(BatchCopyOutcomeItem {
                        source_default_id: id,
                        new_workspace_formula_id: None,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
        Ok(BatchCopySummary {
            items,
            succeeded,
            failed,
        })
    }
}
