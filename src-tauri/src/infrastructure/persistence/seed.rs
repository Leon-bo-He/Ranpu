//! 首次启动种子数据（PROMPT 第 304-308 行）+ 系统内置 "通用" 工作区初始化。
//!
//! 不创建 user — FirstRunSetup 让用户自己设置第一个 admin 账号。
//!
//! - run_if_empty: 老的首次启动种子, 写 3 个示范工作区 + 5 条默认配方.
//! - ensure_system_mirror: 确保数据库里存在唯一的 SystemMirror 工作区
//!   ("通用"), 并把所有默认配方镜像到该工作区. 每次启动都跑, 用于
//!   1) 第一次创建该工作区, 2) 老 DB 迁移, 3) 自愈漂移.

use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::application::ports::default_formula_repository::{
    DefaultFormulaQuery, DefaultFormulaRepository,
};
use crate::application::ports::errors::RepositoryError;
use crate::application::ports::workspace_formula_repository::{
    WorkspaceFormulaQuery, WorkspaceFormulaRepository,
};
use crate::application::ports::workspace_repository::WorkspaceRepository;
use crate::domain::formula::amounts::Kilograms;
use crate::domain::formula::default_formula::DefaultFormula;
use crate::domain::formula::formula_item::FormulaItem;
use crate::domain::formula::internal_color_code::InternalColorCode;
use crate::domain::formula::liquor_ratio::LiquorRatio;
use crate::domain::formula::unit::Unit;
use crate::domain::workspace::workspace::{Workspace, WorkspaceKind, WorkspaceName};

pub const SYSTEM_MIRROR_WORKSPACE_NAME: &str = "通用";

pub fn run_if_empty(
    workspace_repo: &Arc<dyn WorkspaceRepository>,
    default_repo: &Arc<dyn DefaultFormulaRepository>,
    now: DateTime<Utc>,
) -> Result<bool, RepositoryError> {
    if !workspace_repo.list_all()?.is_empty() {
        return Ok(false);
    }

    seed_workspaces(workspace_repo.as_ref(), now)?;
    seed_default_formulas(default_repo.as_ref(), now)?;
    Ok(true)
}

/// 确保系统内置 "通用" 工作区存在并镜像默认配方库.
///
/// - 没有该工作区 → 创建.
/// - 该工作区里有但默认库已删除的 → 删除.
/// - 默认库新增 / 修改的 → upsert 进该工作区.
pub fn ensure_system_mirror(
    workspace_repo: &Arc<dyn WorkspaceRepository>,
    default_repo: &Arc<dyn DefaultFormulaRepository>,
    workspace_formula_repo: &Arc<dyn WorkspaceFormulaRepository>,
    now: DateTime<Utc>,
) -> Result<(), RepositoryError> {
    let mirror = match workspace_repo.find_system_mirror()? {
        Some(w) => w,
        None => {
            // 名字冲突: 如果用户曾经手动创建过同名 normal 工作区, 改名让位.
            if let Some(conflicting) = workspace_repo.find_by_name(SYSTEM_MIRROR_WORKSPACE_NAME)? {
                if let Some(id) = conflicting.id() {
                    let renamed = format!("{SYSTEM_MIRROR_WORKSPACE_NAME}-用户");
                    workspace_repo.rename(id, &renamed)?;
                }
            }
            let ws = Workspace::new_with_kind(
                WorkspaceName::new(SYSTEM_MIRROR_WORKSPACE_NAME)
                    .map_err(|e| RepositoryError::Backend(e.to_string()))?,
                Some("系统内置工作区, 配方与默认配方库自动同步, 无法直接编辑.".into()),
                now,
                WorkspaceKind::SystemMirror,
            )
            .map_err(|e| RepositoryError::Backend(e.to_string()))?;
            let id = workspace_repo.insert(&ws)?;
            workspace_repo
                .find_by_id(id)?
                .ok_or_else(|| RepositoryError::Backend("system mirror 插入后查不到".into()))?
        }
    };
    let mirror_id = mirror
        .id()
        .ok_or_else(|| RepositoryError::Backend("system mirror 缺少 id".into()))?;

    // 全量同步: 默认库 → 镜像工作区.
    let defaults = default_repo.list(DefaultFormulaQuery {
        keyword: None,
        limit: None,
        offset: None,
    })?;
    let mirror_existing = workspace_formula_repo.list(WorkspaceFormulaQuery {
        workspace_id: mirror_id,
        keyword: None,
        limit: None,
        offset: None,
    })?;

    use std::collections::HashSet;
    let default_codes: HashSet<String> = defaults
        .iter()
        .map(|d| {
            <DefaultFormula as crate::domain::calculation::dye_calculator::CalculableFormula>::internal_color_code(d)
                .as_str()
                .to_owned()
        })
        .collect();

    // 1) 删除镜像里多出来的 (默认库已不存在的)
    for wf in &mirror_existing {
        let code = <crate::domain::formula::workspace_formula::WorkspaceFormula as crate::domain::calculation::dye_calculator::CalculableFormula>::internal_color_code(wf)
            .as_str()
            .to_owned();
        if !default_codes.contains(&code) {
            if let Some(wf_id) = wf.id() {
                workspace_formula_repo.delete(mirror_id, wf_id)?;
            }
        }
    }

    // 2) upsert 默认库每条到镜像
    for d in &defaults {
        upsert_default_into_mirror(workspace_formula_repo.as_ref(), d, mirror_id, now)?;
    }

    Ok(())
}

/// 把一条默认配方刷到镜像工作区: 已存在 → 用其 id 整体覆盖, 否则新插入.
pub fn upsert_default_into_mirror(
    repo: &dyn WorkspaceFormulaRepository,
    default: &DefaultFormula,
    mirror_id: crate::domain::shared::id::WorkspaceId,
    now: DateTime<Utc>,
) -> Result<(), RepositoryError> {
    use crate::domain::calculation::dye_calculator::CalculableFormula;
    use crate::domain::formula::workspace_formula::WorkspaceFormula;

    let internal =
        <DefaultFormula as CalculableFormula>::internal_color_code(default).clone();
    let existing = repo.find_by_internal_code(mirror_id, &internal)?;

    let formula = match existing {
        Some(prev) => WorkspaceFormula::rehydrate(
            prev.id().ok_or_else(|| {
                RepositoryError::Backend("workspace_formula 缺少 id".into())
            })?,
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
        )
        .map_err(|e| RepositoryError::Backend(e.to_string()))?,
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
        )
        .map_err(|e| RepositoryError::Backend(e.to_string()))?,
    };
    repo.upsert(&formula)?;
    Ok(())
}

fn seed_workspaces(
    repo: &dyn WorkspaceRepository,
    now: DateTime<Utc>,
) -> Result<(), RepositoryError> {
    for name in ["客户A", "客户B", "客户C"] {
        let workspace = Workspace::new(
            WorkspaceName::new(name).map_err(|e| RepositoryError::Backend(e.to_string()))?,
            None,
            now,
        )
        .map_err(|e| RepositoryError::Backend(e.to_string()))?;
        repo.insert(&workspace)?;
    }
    Ok(())
}

fn seed_default_formulas(
    repo: &dyn DefaultFormulaRepository,
    now: DateTime<Utc>,
) -> Result<(), RepositoryError> {
    let formulas = vec![
        navy_n_2024(now),
        rose_r_105(now),
        forest_g_330(now),
        sunset_o_710(now),
        ivory_w_002(now),
    ];
    for f in formulas {
        repo.upsert(&f)?;
    }
    Ok(())
}

fn navy_n_2024(now: DateTime<Utc>) -> DefaultFormula {
    DefaultFormula::new(
        InternalColorCode::new("N-2024").unwrap(),
        None,
        Some("藏青".into()),
        Some("偏冷调藏青，适合棉与混纺。".into()),
        Some(Kilograms::new(50.0).unwrap()),
        Some(LiquorRatio::new(8.0).unwrap()),
        Some("活性染料，纯碱固色。".into()),
        vec![
            FormulaItem::new("C.I. Reactive Blue 19", Some("RB19".into()), 2.0, Unit::PctOwf, 0)
                .unwrap(),
            FormulaItem::new("C.I. Reactive Black 5", Some("RB5".into()), 1.5, Unit::PctOwf, 1)
                .unwrap(),
            FormulaItem::new("元明粉", None, 60.0, Unit::GramsPerL, 2).unwrap(),
            FormulaItem::new("纯碱", None, 20.0, Unit::GramsPerL, 3).unwrap(),
        ],
        now,
    )
    .unwrap()
}

fn rose_r_105(now: DateTime<Utc>) -> DefaultFormula {
    DefaultFormula::new(
        InternalColorCode::new("R-105").unwrap(),
        None,
        Some("玫红".into()),
        Some("中等饱和度玫红。".into()),
        Some(Kilograms::new(30.0).unwrap()),
        Some(LiquorRatio::new(10.0).unwrap()),
        None,
        vec![
            FormulaItem::new("C.I. Reactive Red 195", Some("RR195".into()), 1.8, Unit::PctOwf, 0)
                .unwrap(),
            FormulaItem::new(
                "C.I. Reactive Yellow 145",
                Some("RY145".into()),
                0.2,
                Unit::PctOwf,
                1,
            )
            .unwrap(),
            FormulaItem::new("元明粉", None, 50.0, Unit::GramsPerL, 2).unwrap(),
        ],
        now,
    )
    .unwrap()
}

fn forest_g_330(now: DateTime<Utc>) -> DefaultFormula {
    DefaultFormula::new(
        InternalColorCode::new("G-330").unwrap(),
        None,
        Some("墨绿".into()),
        Some("偏黑墨绿。".into()),
        Some(Kilograms::new(40.0).unwrap()),
        Some(LiquorRatio::new(8.0).unwrap()),
        None,
        vec![
            FormulaItem::new(
                "C.I. Reactive Yellow 145",
                Some("RY145".into()),
                0.8,
                Unit::PctOwf,
                0,
            )
            .unwrap(),
            FormulaItem::new(
                "C.I. Reactive Blue 19",
                Some("RB19".into()),
                1.5,
                Unit::PctOwf,
                1,
            )
            .unwrap(),
            FormulaItem::new(
                "C.I. Reactive Black 5",
                Some("RB5".into()),
                0.5,
                Unit::PctOwf,
                2,
            )
            .unwrap(),
            FormulaItem::new("元明粉", None, 55.0, Unit::GramsPerL, 3).unwrap(),
        ],
        now,
    )
    .unwrap()
}

fn sunset_o_710(now: DateTime<Utc>) -> DefaultFormula {
    DefaultFormula::new(
        InternalColorCode::new("O-710").unwrap(),
        None,
        Some("落日橙".into()),
        Some("偏红的鲜橙色，量多需注意 pH。".into()),
        None,
        None,
        None,
        vec![
            FormulaItem::new(
                "C.I. Reactive Orange 16",
                Some("RO16".into()),
                2.5,
                Unit::PctOwf,
                0,
            )
            .unwrap(),
            FormulaItem::new(
                "C.I. Reactive Yellow 145",
                Some("RY145".into()),
                0.5,
                Unit::PctOwf,
                1,
            )
            .unwrap(),
            FormulaItem::new("元明粉", None, 40.0, Unit::GramsPerKg, 2).unwrap(),
        ],
        now,
    )
    .unwrap()
}

fn ivory_w_002(now: DateTime<Utc>) -> DefaultFormula {
    DefaultFormula::new(
        InternalColorCode::new("W-002").unwrap(),
        None,
        Some("象牙白".into()),
        Some("浅米色，染色后接近原棉色。".into()),
        Some(Kilograms::new(20.0).unwrap()),
        None,
        Some("浴比 1:8，建议先染浅黄校色再加红.".into()),
        vec![
            FormulaItem::new(
                "C.I. Reactive Yellow 145",
                Some("RY145".into()),
                0.05,
                Unit::PctOwf,
                0,
            )
            .unwrap(),
            FormulaItem::new(
                "C.I. Reactive Red 195",
                Some("RR195".into()),
                0.02,
                Unit::PctOwf,
                1,
            )
            .unwrap(),
        ],
        now,
    )
    .unwrap()
}
