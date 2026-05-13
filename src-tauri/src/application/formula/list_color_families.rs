//! 列出已用过的 "色系" 字符串, 给前端 dropdown 用.
//!
//! - default 库: 全局 distinct.
//! - workspace: 当前激活工作区 distinct (没激活就报错).

use crate::application::errors::AppResult;
use crate::application::formula::service::FormulaService;
use crate::application::session_guard::{ensure_active, ensure_active_workspace};

impl FormulaService {
    pub fn list_default_color_families(&self) -> AppResult<Vec<String>> {
        ensure_active(&*self.session_store)?;
        Ok(self.default_repo.list_color_families()?)
    }

    pub fn list_workspace_color_families(&self) -> AppResult<Vec<String>> {
        let (_, workspace_id) = ensure_active_workspace(&*self.session_store)?;
        Ok(self.workspace_repo.list_color_families(workspace_id)?)
    }

    /// 跨默认库 + 所有工作区的 distinct 色系, 字典序去重. 前端首次升级到
    /// 有 "色系库" 的版本时一次性导入历史数据.
    pub fn list_all_color_families(&self) -> AppResult<Vec<String>> {
        ensure_active(&*self.session_store)?;
        let mut seen = std::collections::BTreeSet::new();
        for c in self.default_repo.list_color_families()? {
            seen.insert(c);
        }
        for c in self.workspace_repo.list_all_color_families()? {
            seen.insert(c);
        }
        Ok(seen.into_iter().collect())
    }
}
