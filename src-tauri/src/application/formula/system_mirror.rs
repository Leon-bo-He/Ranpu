//! 系统内置 "通用" 工作区与默认配方库之间的同步辅助.
//!
//! 默认配方 upsert / delete 之后调一下这里的方法, 把变更刷到 system_mirror 工作区.
//! 没有镜像工作区时静默跳过 — 启动时 ensure_system_mirror 会自愈漂移.

use chrono::{DateTime, Utc};

use crate::application::errors::AppResult;
use crate::application::formula::service::FormulaService;
use crate::domain::calculation::dye_calculator::CalculableFormula;
use crate::domain::formula::default_formula::DefaultFormula;
use crate::domain::formula::internal_color_code::InternalColorCode;
use crate::domain::formula::workspace_formula::WorkspaceFormula;
use crate::domain::shared::id::WorkspaceId;

impl FormulaService {
    pub(super) fn mirror_default_upsert(
        &self,
        default: &DefaultFormula,
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        if let Some(mirror_id) = self.system_mirror_id()? {
            upsert_into_mirror(self, default, mirror_id, now)?;
        }
        Ok(())
    }

    pub(super) fn mirror_default_delete_by_internal(
        &self,
        internal: &InternalColorCode,
    ) -> AppResult<()> {
        if let Some(mirror_id) = self.system_mirror_id()? {
            if let Some(existing) = self
                .workspace_repo
                .find_by_internal_code(mirror_id, internal)?
            {
                if let Some(wf_id) = existing.id() {
                    self.workspace_repo.delete(mirror_id, wf_id)?;
                }
            }
        }
        Ok(())
    }

    fn system_mirror_id(&self) -> AppResult<Option<WorkspaceId>> {
        Ok(self
            .workspaces_repo
            .find_system_mirror()?
            .and_then(|w| w.id()))
    }
}

fn upsert_into_mirror(
    svc: &FormulaService,
    default: &DefaultFormula,
    mirror_id: WorkspaceId,
    now: DateTime<Utc>,
) -> AppResult<()> {
    let internal = <DefaultFormula as CalculableFormula>::internal_color_code(default).clone();
    let existing = svc
        .workspace_repo
        .find_by_internal_code(mirror_id, &internal)?;

    let formula = match existing {
        Some(prev) => WorkspaceFormula::rehydrate(
            prev.id().expect("persisted"),
            mirror_id,
            internal,
            default.customer_color_code().cloned(),
            default.color_name().map(str::to_owned),
            default.description().map(str::to_owned),
            default.base_weight_kg(),
            <DefaultFormula as CalculableFormula>::liquor_ratio(default),
            default.notes().map(str::to_owned),
            <DefaultFormula as CalculableFormula>::items(default).to_vec(),
            default.id(),
            prev.created_at(),
            now,
        )?,
        None => WorkspaceFormula::new(
            mirror_id,
            internal,
            default.customer_color_code().cloned(),
            default.color_name().map(str::to_owned),
            default.description().map(str::to_owned),
            default.base_weight_kg(),
            <DefaultFormula as CalculableFormula>::liquor_ratio(default),
            default.notes().map(str::to_owned),
            <DefaultFormula as CalculableFormula>::items(default).to_vec(),
            default.id(),
            now,
        )?,
    };
    svc.workspace_repo.upsert(&formula)?;
    Ok(())
}
