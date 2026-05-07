//! 首次启动种子数据（PROMPT 第 304-308 行）。
//!
//! 不创建 user — FirstRunSetup 让用户自己设置第一个 admin 账号。
//! 创建：3 个 workspace + 5 条 default 配方（带真实风格染料组合）。

use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::application::ports::default_formula_repository::DefaultFormulaRepository;
use crate::application::ports::errors::RepositoryError;
use crate::application::ports::workspace_repository::WorkspaceRepository;
use crate::domain::formula::amounts::Kilograms;
use crate::domain::formula::customer_color_code::CustomerColorCode;
use crate::domain::formula::default_formula::DefaultFormula;
use crate::domain::formula::formula_item::FormulaItem;
use crate::domain::formula::internal_color_code::InternalColorCode;
use crate::domain::formula::liquor_ratio::LiquorRatio;
use crate::domain::formula::unit::Unit;
use crate::domain::workspace::workspace::{Workspace, WorkspaceName};

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

fn seed_workspaces(
    repo: &dyn WorkspaceRepository,
    now: DateTime<Utc>,
) -> Result<(), RepositoryError> {
    for name in ["客户A", "客户B", "客户C"] {
        let workspace = Workspace::new(
            WorkspaceName::new(name).map_err(|e| RepositoryError::Backend(e.to_string()))?,
            None,
            None, // 系统种子，无创建者；DB 列允许 NULL。
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
        Some(CustomerColorCode::new("CUST-NAVY-01").unwrap()),
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
        None,
        now,
    )
    .unwrap()
}

fn rose_r_105(now: DateTime<Utc>) -> DefaultFormula {
    DefaultFormula::new(
        InternalColorCode::new("R-105").unwrap(),
        Some(CustomerColorCode::new("CUST-ROSE-12").unwrap()),
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
        None,
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
        None,
        now,
    )
    .unwrap()
}

fn sunset_o_710(now: DateTime<Utc>) -> DefaultFormula {
    DefaultFormula::new(
        InternalColorCode::new("O-710").unwrap(),
        Some(CustomerColorCode::new("CUST-ORANGE-7").unwrap()),
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
        None,
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
        None,
        now,
    )
    .unwrap()
}
