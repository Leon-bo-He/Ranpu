//! 解密 + 解析 .ranpu 归档, 仅做预览不写库.
//!
//! UI 在用户提交导入选项前调一次, 用于:
//!   1. 显示归档里有多少默认配方 / 多少工作区 / 各工作区多少配方
//!   2. 按工作区名称在目标库匹配, 告知 UI 哪些工作区是 "已存在 (需 merge/skip)"
//!      vs "新建"

use std::path::PathBuf;

use crate::application::errors::{AppError, AppResult};
use crate::application::formula::service::FormulaService;
use crate::application::formula::wire::{
    FormulaArchive, FORMULA_ARCHIVE_MAGIC, FORMULA_ARCHIVE_VERSION,
};
use crate::application::session_guard::ensure_active;

#[derive(Debug, Clone)]
pub struct PreviewArchiveInput {
    pub passphrase: String,
    pub in_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PreviewArchive {
    pub exported_at: String,
    pub default_count: u32,
    pub has_default: bool,
    pub workspaces: Vec<PreviewWorkspace>,
}

#[derive(Debug, Clone)]
pub struct PreviewWorkspace {
    pub name: String,
    pub description: Option<String>,
    pub formula_count: u32,
    /// true → 目标库已存在同名工作区 (需 UI 选 merge / skip);
    /// false → 目标库没有, 导入会新建.
    pub already_exists: bool,
}

impl FormulaService {
    pub fn preview_library_archive(
        &self,
        input: PreviewArchiveInput,
    ) -> AppResult<PreviewArchive> {
        ensure_active(&*self.session_store)?;

        let bytes = self
            .encrypted_importer
            .import_from_file(&input.in_path, &input.passphrase)
            .map_err(|e| AppError::Crypto(e.to_string()))?;

        let payload: FormulaArchive = serde_json::from_slice(&bytes)
            .map_err(|e| AppError::Internal(format!("解析配方归档 JSON 失败：{e}")))?;
        if payload.magic != FORMULA_ARCHIVE_MAGIC {
            return Err(AppError::Internal(
                "文件签名不匹配，可能不是染谱配方导出文件".into(),
            ));
        }
        if payload.version != FORMULA_ARCHIVE_VERSION {
            return Err(AppError::Internal(format!(
                "不支持的归档版本：{}",
                payload.version
            )));
        }

        let mut workspaces = Vec::with_capacity(payload.workspaces.len());
        for ws in &payload.workspaces {
            let already_exists = self.workspaces_repo.find_by_name(&ws.name)?.is_some();
            workspaces.push(PreviewWorkspace {
                name: ws.name.clone(),
                description: ws.description.clone(),
                formula_count: ws.formulas.len() as u32,
                already_exists,
            });
        }

        Ok(PreviewArchive {
            exported_at: payload.exported_at,
            default_count: payload.default_formulas.len() as u32,
            has_default: !payload.default_formulas.is_empty(),
            workspaces,
        })
    }
}
